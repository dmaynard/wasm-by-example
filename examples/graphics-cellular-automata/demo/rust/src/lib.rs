// The wasm-pack uses wasm-bindgen to build and generate JavaScript binding file.
// Import the wasm-bindgen crate.
use wasm_bindgen::prelude::*;

// I had trouble getting the Rust Rand crate to work in WASM
// so instead I will use the ES6 random number generator
// the sy_sys crate provide rust access to many ES6 system calls.
// js_sys::Math::random() is the only one used here.
extern crate js_sys;

// Define the size of our "GRID"
const MAX_GRID_W: u32 = 2000;
const MAX_GRID_H: u32 = 2000;
const MAX_GRID_SIZE: u32 = MAX_GRID_W * MAX_GRID_H;

/*
 * 1. What is going on here?
 * WE create a static mutable buffer.
 * We will use for putting the output of our graphics,
 * to pass the output to js.
 * NOTE: global `static mut` means we will have "unsafe" code
 * but for passing memory between js and wasm should be fine
 * since this code is single threaded.
 *
 * 2. In this example we treat the OUTPUT_BUFFER as an array of
 * 32 bit pixels [u32: GRID_SIZE] (0xAABBGGRR) and store each
 * entire pixel with a single u32 store.
 */
const OUTPUT_BUFFER_SIZE: usize = (MAX_GRID_SIZE * 4) as usize;
static mut OUTPUT_BUFFER: [u8; OUTPUT_BUFFER_SIZE] = [0; OUTPUT_BUFFER_SIZE];

static mut OLD_STATE: [u8; MAX_GRID_SIZE as usize] = [0; MAX_GRID_SIZE as usize];
static mut NEW_STATE: [u8; MAX_GRID_SIZE as usize] = [0; MAX_GRID_SIZE as usize];

const NUM_STATES: u8 = 16;

const PALETTE: [(u8, u8, u8, u8); NUM_STATES as usize] = [
    // (R,G,B,A)
    (0xff, 0xff, 0xff, 0xff),
    (0xee, 0xee, 0xee, 0xff),
    (0xdd, 0xdd, 0xdd, 0xff),
    (0xcc, 0xcc, 0xcc, 0xff),
    (0xbb, 0xbb, 0xbb, 0xff),
    (0xaa, 0xaa, 0xaa, 0xff),
    (0x99, 0x99, 0x99, 0xff),
    (0x88, 0x88, 0x88, 0xff),
    (0x77, 0x77, 0x77, 0xff),
    (0x66, 0x66, 0x66, 0xff),
    (0x55, 0x55, 0x55, 0xff),
    (0x44, 0x44, 0x44, 0xff),
    (0x33, 0x33, 0x33, 0xff),
    (0x22, 0x22, 0x22, 0xff),
    (0x11, 0x11, 0x11, 0xff),
    (0x00, 0x00, 0x00, 0xff),
];
const MATERIAL_PALETTE: [(u8, u8, u8, u8); NUM_STATES as usize] = [
    //(R,G,B,A)
    (244, 67, 54, 255),
    (232, 30, 99, 255),
    (156, 39, 176, 255),
    (103, 58, 183, 255),
    (63, 81, 181, 255),
    (33, 150, 243, 255),
    (3, 169, 244, 255),
    (0, 188, 212, 255),
    (0, 150, 136, 255),
    (76, 175, 80, 255),
    (139, 195, 74, 255),
    (205, 220, 57, 255),
    (255, 235, 59, 255),
    (255, 193, 7, 255),
    (255, 152, 0, 255),
    (255, 87, 34, 255),
];

// Function to return a pointer to our buffer
// in wasm memory
#[wasm_bindgen]
pub fn get_output_buffer_pointer() -> *const u8 {
    let pointer: *const u8;
    unsafe {
        pointer = OUTPUT_BUFFER.as_ptr();
    }
    pointer
}
fn get_idx(x: u32, y: u32, w: u32) -> usize {
    match (y * w + x).try_into() {
        Ok(idx) => idx,
        Err(why) => panic!("{why:?}"),
    }
}
fn get_neighbors(x: u32, y: u32, w: u32, h: u32) -> [usize; 4] {
    [
        // left
        get_idx(if x == 0 { w - 1 } else { (x - 1) % w }, y, w),
        // right
        get_idx((x + 1) % w, y, w),
        // above
        get_idx(x, if y == 0 { h - 1 } else { (y - 1) % h }, w),
        // below
        get_idx(x, (y + 1) % h, w),
    ]
}
// Function to generate the next generation of our crystal, pixel by pixel
#[wasm_bindgen]
pub fn update_crystal(init: bool, color: bool, width: u32, height: u32) -> u32 {
    // Since Linear memory is a 1 dimensional array, but we want a grid
    // we will be doing 2d to 1d mapping
    // https://softwareengineering.stackexchange.com/questions/212808/treating-a-1d-data-structure-as-2d-grid
    let mut n_deltas: u32 = 4;

    let the_palette = if color { &MATERIAL_PALETTE } else { &PALETTE };
    if init {
        for y in 0..height {
            for x in 0..width {
                let state: u8 = (js_sys::Math::random() * NUM_STATES as f64) as u8;
                let idx: usize = ((y * width) + x) as usize;
                unsafe {
                    NEW_STATE[idx] = state;
                    OLD_STATE[idx] = state;
                }
            }
        }
    }
    // update the new state from the old
    for y in 0..height {
        for x in 0..width {
            let idx: usize = ((y * width) + x) as usize;
            let neighbors: [usize; 4] = get_neighbors(x, y, width, height);
            unsafe {
                let cur_cell = OLD_STATE[idx];
                for n_idx in neighbors {
                    // if any neighbor is one state higher it eats this cell
                    if OLD_STATE[n_idx] == (cur_cell + 1) % NUM_STATES {
                        NEW_STATE[idx] = OLD_STATE[n_idx];
                        n_deltas += 1;
                        break;
                    }
                }
            }
        }
    }
    // map from state to color
    for y in 0..height {
        for x in 0..width {
            let idx: usize = get_idx(x, y, width);
            let byte_idx = idx * 4;
            unsafe {
                OUTPUT_BUFFER[byte_idx] = the_palette[(NEW_STATE[idx]) as usize].0;
                OUTPUT_BUFFER[byte_idx + 1] = the_palette[(NEW_STATE[idx]) as usize].1;
                OUTPUT_BUFFER[byte_idx + 2] = the_palette[(NEW_STATE[idx]) as usize].2;
                OUTPUT_BUFFER[byte_idx + 3] = the_palette[(NEW_STATE[idx]) as usize].3;
            }
        }
    }
    // new now becomes old
    for y in 0..height {
        for x in 0..width {
            let idx: usize = get_idx(x, y, width);
            unsafe {
                OLD_STATE[idx] = NEW_STATE[idx];
            }
        }
    }
    // return the number of pixels updated
    // the Javascript code uses this to determine when to
    // start a new crystal
    n_deltas
}
