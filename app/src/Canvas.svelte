<script>
  import { onMount } from "svelte";

  export let userSelectedColor;
  let canvas, ctx;
  let isDrawing = false;
  let lastX = 0;
  let lastY = 0;

  // Initialize the canvas context
  function initCanvas() {
    console.log("initCanvas");
    ctx = canvas.getContext("2d");
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;

    // Set background to white
    ctx.fillStyle = "white";
    ctx.fillRect(0, 0, canvas.width, canvas.height);
  }

  // Resize the canvas non-destructively
  // Note: resizing the canvas will clear the canvas context
  // that is larger than the canvas itself (i.e. off-screen is cleared)
  function resizeCanvas() {
    console.log("resizeCanvas");
    const tempCanvas = document.createElement("canvas");
    const tempCtx = tempCanvas.getContext("2d");
    tempCanvas.width = window.innerWidth;
    tempCanvas.height = window.innerHeight;
    tempCtx.drawImage(canvas, 0, 0);
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;
    // Reapply white background
    ctx.fillStyle = "white";
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    ctx.drawImage(tempCanvas, 0, 0);
  }

  // Start drawing
  function startDrawing(e) {
    isDrawing = true;
    [lastX, lastY] = [e.clientX, e.clientY];
  }

  // Draw on canvas
  function draw(e) {
    if (!isDrawing) return;
    ctx.beginPath();
    ctx.moveTo(lastX, lastY);
    ctx.lineTo(e.clientX, e.clientY);
    // add color
    ctx.strokeStyle = userSelectedColor;
    ctx.lineWidth = 6;
    ctx.lineCap = "round";

    ctx.stroke();
    [lastX, lastY] = [e.clientX, e.clientY];
  }

  // Stop drawing
  function stopDrawing() {
    isDrawing = false;
  }

  // Initialize the canvas context
  onMount(() => {
    initCanvas();

    // listen for window resize
    window.addEventListener("resize", resizeCanvas);
  });
</script>

<canvas
  bind:this={canvas}
  on:mousedown={startDrawing}
  on:mousemove={draw}
  on:mouseup={stopDrawing}
  on:mouseout={stopDrawing}
  on:resize={initCanvas}
  on:blur={stopDrawing}
/>

<style>
  canvas {
    position: absolute;
    top: 0;
    left: 0;
    background: var(--canvas-background);

    cursor: url("data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg'  width='128' height='80' viewport='0 0 100 100' style='fill:black;font-size:300px;'>  <path stroke-linecap=\"round\" stroke-linejoin=\"round\" d=\"M16.862 4.487l1.687-1.688a1.875 1.875 0 112.652 2.652L6.832 19.82a4.5 4.5 0 01-1.897 1.13l-2.685.8.8-2.685a4.5 4.5 0 011.13-1.897L16.863 4.487zm0 0L19.5 7.125\" /></svg>") 0 22, auto;
  }
</style>
