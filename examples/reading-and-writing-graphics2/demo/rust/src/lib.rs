// The wasm-pack uses wasm-bindgen to build and generate JavaScript binding file.
// Import the wasm-bindgen crate.
extern crate js_sys;
use wasm_bindgen::prelude::*;

// Define the size of our "GRID"
const GRID_W: usize = 100;
const GRID_H: usize = 100;
const GRID_SIZE: usize = GRID_W * GRID_H;

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
const OUTPUT_BUFFER_SIZE: usize = GRID_SIZE;
static mut OUTPUT_BUFFER: [u32; OUTPUT_BUFFER_SIZE] = [0; OUTPUT_BUFFER_SIZE];

static mut OLD_STATE: [u8; GRID_SIZE] = [0; GRID_SIZE];
static mut NEW_STATE: [u8; GRID_SIZE] = [0; GRID_SIZE];

const NUM_STATES: u8 = 8;

const PALETTE: [u32; NUM_STATES as usize] = [
    // AABBGGRR
    0xffffffff, 0xffdddddd, 0xffbbbbbb, 0xff999999, 0xff777777, 0xff555555, 0xff333333, 0xff111111,
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

// Function to generate our checkerboard, pixel by pixel
#[wasm_bindgen]
pub fn update_crystal(init: bool) {
    // Since Linear memory is a 1 dimensional array, but we want a grid
    // we will be doing 2d to 1d mapping
    // https://softwareengineering.stackexchange.com/questions/212808/treating-a-1d-data-structure-as-2d-grid
    if init {
        for y in 0..GRID_H {
            for x in 0..GRID_W {
                let state: u8 = (js_sys::Math::random() * NUM_STATES as f64) as u8;

                unsafe {
                    NEW_STATE[y * GRID_W + x] = state;
                }
            }
        }
    }
    // map from state to color
    for y in 0..GRID_H {
        for x in 0..GRID_W {
            unsafe {
                OUTPUT_BUFFER[y * GRID_W + x] = PALETTE[(NEW_STATE[y * GRID_W + x]) as usize];
            }
        }
    }
    // map from state to color
    for y in 0..GRID_H {
        for x in 0..GRID_W {
            unsafe {
                OLD_STATE[y * GRID_W + x] = NEW_STATE[y * GRID_W + x];
            }
        }
    }
}
