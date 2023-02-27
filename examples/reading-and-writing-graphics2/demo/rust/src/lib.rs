// The wasm-pack uses wasm-bindgen to build and generate JavaScript binding file.
// Import the wasm-bindgen crate.
extern crate js_sys;
use wasm_bindgen::prelude::*;

// Define the size of our "GRID"
const GRID_W: u32 = 100;
const GRID_H: u32 = 100;
const GRID_SIZE: u32 = GRID_W * GRID_H;

/*
 * 1. What is going on here?
 * Create a static mutable byte buffer.
 * We will use for putting the output of our graphics,
 * to pass the output to js.
 * NOTE: global `static mut` means we will have "unsafe" code
 * but for passing memory between js and wasm should be fine.
 *
 * 2. Why is the size CHECKERBOARD_SIZE * CHECKERBOARD_SIZE * 4?
 * We want to have 20 pixels by 20 pixels. And 4 colors per pixel (r,g,b,a)
 * Which, the Canvas API Supports.
 */
const OUTPUT_BUFFER_SIZE: u32 = GRID_SIZE;
static mut OUTPUT_BUFFER: [u32; OUTPUT_BUFFER_SIZE as usize] = [0; OUTPUT_BUFFER_SIZE as usize];

static mut OLD_STATE: [u8; GRID_SIZE as usize] = [0; GRID_SIZE as usize];
static mut NEW_STATE: [u8; GRID_SIZE as usize] = [0; GRID_SIZE as usize];

const NUM_STATES: u8 = 16;

const PALETTE: [u32; NUM_STATES as usize] = [
    // AABBGGRR
    0xffffffff, 0xffeeeeee, 0xffdddddd, 0xffcccccc, 0xffbbbbbb, 0xffaaaaaa, 0xff999999, 0xff888888,
    0xff777777, 0xff666666, 0xff555555, 0xff444444, 0xff333333, 0xff222222, 0xff111111, 0xff000000,
];
// Function to return a pointer to our buffer
// in wasm memory
#[wasm_bindgen]
pub fn get_output_buffer_pointer() -> *const u32 {
    let pointer: *const u32;
    unsafe {
        pointer = OUTPUT_BUFFER.as_ptr();
    }

    return pointer;
}
fn get_idx(x: u32, y: u32) -> usize {
    match (y * GRID_W + x).try_into() {
        Ok(idx) => idx,
        Err(why) => panic!("{:?}", why),
    }
}
fn get_neighbors(x: u32, y: u32) -> [usize; 4] {
    [
        // left
        get_idx(if x == 0 { GRID_W - 1 } else { (x - 1) % GRID_W }, y),
        // right
        get_idx((x + 1) % GRID_W, y),
        // above
        get_idx(x, if y == 0 { GRID_H - 1 } else { (y - 1) % GRID_H }),
        // below
        get_idx(x, (y + 1) % GRID_H),
    ]
}
// Function to generate our crystal, pixel by pixel
#[wasm_bindgen]
pub fn update_crystal(init: bool) {
    // Since Linear memory is a 1 dimensional array, but we want a grid
    // we will be doing 2d to 1d mapping
    // https://softwareengineering.stackexchange.com/questions/212808/treating-a-1d-data-structure-as-2d-grid
    if init {
        for y in 0..GRID_H {
            for x in 0..GRID_W {
                let state: u8 = (js_sys::Math::random() * NUM_STATES as f64) as u8;
                let idx: usize = get_idx(x, y);
                unsafe {
                    NEW_STATE[idx] = state;
                    OLD_STATE[idx] = state;
                }
            }
        }
    }
    // update the new state fro the old
    for y in 0..GRID_H {
        for x in 0..GRID_W {
            let idx: usize = get_idx(x, y);
            let neighbors: [usize; 4] = get_neighbors(x, y);
            unsafe {
                let cur_cell = OLD_STATE[idx];
                for n_idx in neighbors {
                    if OLD_STATE[n_idx] == (cur_cell + 1) % NUM_STATES {
                        NEW_STATE[idx] = OLD_STATE[n_idx];
                    }
                }
            }
        }
    }
    // map from state to color
    for y in 0..GRID_H {
        for x in 0..GRID_W {
            let idx: usize = get_idx(x, y);
            unsafe {
                OUTPUT_BUFFER[idx] = PALETTE[(NEW_STATE[idx]) as usize];
            }
        }
    }
    // new now becomes old
    for y in 0..GRID_H {
        for x in 0..GRID_W {
            let idx: usize = get_idx(x, y);
            unsafe {
                OLD_STATE[idx] = NEW_STATE[idx];
            }
        }
    }
}