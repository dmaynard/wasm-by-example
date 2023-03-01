import wasmInit from "./pkg/crystalize.js";

const runWasm = async () => {
  // Instantiate our wasm module
  const rustWasm = await wasmInit("./pkg/crystalize_bg.wasm");

  // Get our canvas element from our index.html
  const canvasElement = document.querySelector("canvas");

  // Set up Context and ImageData on the canvas
  const canvasContext = canvasElement.getContext("2d");
  const canvasImageData = canvasContext.createImageData(
    canvasElement.width,
    canvasElement.height
  );

  // Clear the canvas
  canvasContext.clearRect(0, 0, canvasElement.width, canvasElement.height);
  // We want to stop the reaction when either no pixels change,
  // or the pattern stablizes.
  let restart = false;
  let last_ndelta = 0;
  let streak = 0;
  let paused = false;
  let delayms = 128;
   document.addEventListener('keyup', (event) => {
    var name = event.key;
    var code = event.code;
    if (event.code == "Space" ) paused = !paused;
    if (event.key == "r" ) restart = true;
    if (paused && event.key == ".") drawCrystal(false);
    if (!paused && event.key == "f") {
      delayms = delayms >= 32 ? delayms / 2 : delayms;
      clearInterval(interval);
      interval = setInterval(() => {
        console.log(" delayms: ", delayms)
        if (!paused) drawCrystal(restart);
      }, delayms);
    };

    if (!paused && event.key == "s") {
      delayms = delayms <= 2048  ? delayms * 2 : delayms;
      clearInterval(interval);
      interval = setInterval(() => {
        console.log(" delayms: ", delayms)
        if (!paused) drawCrystal(restart);
      }, delayms);
    }
    console.log(' key: %s code %s', name,code, paused, delayms );
  }, false);

  const drawCrystal = ( init ) => {
    const crtstalSize = 100;

    // Generate a new checkboard in wasm
    let n_deltas = rustWasm.update_crystal(
      init
    )
   
    if (n_deltas == last_ndelta) {
      streak += 1;
    } else {
      streak = 0;
      last_ndelta = n_deltas;
    }

    restart = (n_deltas == 0) || (streak > 200);
   
    // Create a Uint8Array to give us access to Wasm Memory
    const wasmByteMemoryArray = new Uint8Array(rustWasm.memory.buffer);

    // Pull out the RGBA values from Wasm memory
    // Starting at the memory index of out output buffer (given by our pointer)
    // 20 * 20 * 4 = checkboard max X * checkerboard max Y * number of pixel properties (R,G.B,A)
    const outputPointer = rustWasm.get_output_buffer_pointer();
    const imageDataArray = wasmByteMemoryArray.slice(
      outputPointer,
      outputPointer + crtstalSize * crtstalSize * 4
    );

    // Set the values to the canvas image data
    canvasImageData.data.set(imageDataArray);

    // Clear the canvas
    canvasContext.clearRect(0, 0, canvasElement.width, canvasElement.height);

    // Place the new generated checkerboard onto the canvas
    canvasContext.putImageData(canvasImageData, 0, 0);

  };

  // Lastly, call our function to draw the crystal
  // And run this ten times every second
  drawCrystal(true);
  let interval = setInterval(() => {
    console.log(" delayms: ", delayms)
    if (!paused) drawCrystal(restart);
  }, delayms);
};
runWasm();
