<script>
  import { onMount } from "svelte";

  export let clickThrough = false;
  export let onSelectionDrawn;
  export let clear;

  let canvas, ctx;
  let isDrawing = false;
  let startX = 0;
  let startY = 0;
  let drawMade = false;

  let lastClear = 0;
  let fillStyle = "rgba(0, 0, 0, 0.4)";

  // when clear is called call this function
  $: if (lastClear < clear) {
    if (ctx) {
      console.log("clearing canvas");

      // Clear the canvas
      ctx.clearRect(0, 0, canvas.width, canvas.height);

      // Draw the overlay on the whole canvas
      ctx.fillStyle = fillStyle;
      ctx.fillRect(0, 0, canvas.width, canvas.height);

      lastClear = clear;
    }
  }

  // Initialize the canvas context
  function initCanvas() {
    ctx = canvas.getContext("2d");
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;

    // Draw the overlay on the whole canvas
    ctx.fillStyle = fillStyle;
    ctx.fillRect(0, 0, canvas.width, canvas.height);
  }

  // Resize the canvas non-destructively
  function resizeCanvas() {
    const tempCanvas = document.createElement("canvas");
    const tempCtx = tempCanvas.getContext("2d");
    tempCanvas.width = window.innerWidth;
    tempCanvas.height = window.innerHeight;
    tempCtx.drawImage(canvas, 0, 0);
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;
    ctx.drawImage(tempCanvas, 0, 0);
  }

  // Start drawing
  function startDrawing(e) {
    isDrawing = true;
    [startX, startY] = [e.clientX, e.clientY];
  }

  // Draw on canvas
  function draw(e) {
    if (!isDrawing) return;
    drawMade = true;

    const rectWidth = e.clientX - startX;
    const rectHeight = e.clientY - startY;

    // Clear the canvas before drawing
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    // Save the current context state
    ctx.save();

    // draw a rectangle using a path
    ctx.beginPath();
    let rect = new Path2D();
    rect.rect(0, 0, canvas.width, canvas.height);

    // a secon rect that overlaps the first using the first "rect"
    rect.rect(startX, startY, rectWidth, rectHeight);
    ctx.fill(rect, "evenodd");
  }

  // Stop drawing
  function stopDrawing(e) {
    isDrawing = false;
    if (drawMade) {
      onSelectionDrawn({
        startX,
        startY,
        width: e.clientX - startX,
        height: e.clientY - startY,
      });
    }
    drawMade = false;
  }

  // Initialize the canvas context
  onMount(() => {
    initCanvas();
    window.addEventListener("resize", resizeCanvas);
  });
</script>

<canvas
  bind:this={canvas}
  on:mousedown={startDrawing}
  on:mousemove={draw}
  on:mouseup={stopDrawing}
  on:mouseout={stopDrawing}
  on:blur={() => {}}
  style="pointer-events: {clickThrough
    ? 'none'
    : 'auto'}; display: {clickThrough ? 'none' : 'block'}"
/>

<style>
  canvas {
    position: absolute;
    top: 0;
    left: 0;
    background: transparent;
  }
</style>
