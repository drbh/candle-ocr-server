<script>
  import { onMount } from "svelte";
  import {
      ArrowUpTray,
      Bolt,
      ChatBubbleLeftRight,
      Icon,
      Pencil,
      RocketLaunch,
      Trash,
      ViewfinderCircle
  } from "svelte-hero-icons";
  import Canvas from "./Canvas.svelte";
  import SelectCanvas from "./SelectCanvas.svelte";

  let img = "";
  let preview;
  let runtime = "-";
  let output = "";
  let status = "waiting...";
  let textContent = "waiting for a drawing";
  let drawMode = true;
  let currentSelection;
  let clear = 0;
  let userSelectedColor = "#000000"

  // Reset output
  function resetOutput() {
    output = "";
    runtime = "";
    status = "";

    // reset the file input
    const fileField = document.getElementById("file");
    fileField.value = "";
  }

  // Toggle between draw and select mode
  function toggleDrawMode() {
    drawMode = !drawMode;
    currentSelection = null;

    // toggle the cursor
    const page = document.querySelector("html");
    if (!drawMode) {
      page.style.cursor = "crosshair";
    } else {
      page.style.cursor = "default";
    }
  }

  function clearSelection() {
    clear = clear + 1;
    currentSelection = null;
  }

  function onSelectionDrawn(selection) {
    currentSelection = selection;
  }

  function previewFile(event) {
    // toggle to select mode if not already
    if (drawMode) {
      toggleDrawMode();
    }

    console.log("previewFile", event);

    // read the file
    // const fileField = document.getElementById("file");
    const file = event.target.files[0];
    const reader = new FileReader();

    // Event handler executed when the file has been read
    reader.onload = function (e) {
      console.log("reader.onload");

      // add the image to the canvas
      const img = new Image();
      img.src = e.target.result;

      // if image doesnt fit in canvas, resize it based on the smaller dimension
      img.onload = function () {
        console.log("img.onload");
        const canvas = document.querySelector("canvas");
        const ctx = canvas.getContext("2d");

        // get the smaller dimension
        let ratio = 1;
        if (img.width > img.height) {
          ratio = canvas.width / img.width;
        } else {
          ratio = canvas.height / img.height;
        }

        // resize the image
        img.width = img.width * ratio * 0.9;
        img.height = img.height * ratio * 0.9;

        // get center of canvas
        const centerX = canvas.width / 2;
        const centerY = canvas.height / 2;

        // get offset of image
        const offsetX = centerX - img.width / 2;
        const offsetY = centerY - img.height / 2;

        // draw image
        ctx.drawImage(img, offsetX, offsetY, img.width, img.height);
      };
    };

    // Read the file as DataURL
    reader.readAsDataURL(file);
  }

  function uploadImage(event) {
    console.log("uploadImage");
  }

  function clearCanvas() {
    const canvas = document.querySelector("canvas");
    const ctx = canvas.getContext("2d");
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    // Set background to white
    ctx.fillStyle = "white";
    ctx.fillRect(0, 0, canvas.width, canvas.height);
  }

  function submitDrawingAsImage(e) {
    // unfocus the button
    if (e) {
      let elem = e.target;
      elem.blur();
    }

    console.log("submitDrawingAsImage");
    const canvas = document.querySelector("canvas");

    let newCanvas;
    if (currentSelection) {
      // use the selection to get a subset of the canvas
      const ctx = canvas.getContext("2d");
      const imageData = ctx.getImageData(
        currentSelection.startX,
        currentSelection.startY,
        currentSelection.width,
        currentSelection.height
      );

      // create a new canvas with the selection
      newCanvas = document.createElement("canvas");
      newCanvas.width = currentSelection.width;
      newCanvas.height = currentSelection.height;

      const newCtx = newCanvas.getContext("2d");
      newCtx.putImageData(imageData, 0, 0);

      console.log("Made new canvas with selection");
    } else {
      newCanvas = getDrawing(canvas);
      console.log("Made new canvas with drawing");
    }

    // get the drawing from the canvas that is within the selection

    // get image url from canvas
    img = newCanvas
      .toDataURL("image/png")
      .replace("data:image/png;base64,", "");

    uploadFile(newCanvas);

    // Reset animation
    preview.classList.remove("animate-preview");
    // Set display to block
    preview.style.display = "block";

    // keep the same ratio as the newCanvas but try to make it as close to 200px as possible
    let ratio = newCanvas.width / newCanvas.height;
    preview.style.width = "200px";
    preview.style.height = `${200 / ratio}px`;

    void preview.offsetWidth; // Trigger a reflow to restart animation
    preview.classList.add("animate-preview");
  }

  function onSpaceKeydown(e) {
    if (e.code === "Space") {
      toggleDrawMode();
    }
  }

  function onCKeydown(e) {
    if (e.code === "KeyC") {
      clearCanvas();
      clearSelection();
      resetOutput();
    }
  }

  function onEnterKeydown(e) {
    if (e.code === "Enter") {
      submitDrawingAsImage();
    }
  }

  // on F open click the file input
  function onFKeydown(e) {
    if (e.code === "KeyF") {
      clickFileInput();
    }
  }

  // on s toggle draw mode
  // function onSKeydown(e) {
  //   if (e.code === "KeyS") {
  //     toggleDrawMode();
  //   }
  // }

  // Function to grab only the drawing from the canvas
  function getDrawing(canvas) {
    // subset the canvas to the drawing area
    const ctx = canvas.getContext("2d");
    const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);
    const data = imageData.data;

    // iterate over all pixels to find the min and max x and y
    let minX = canvas.width;
    let maxX = 0;
    let minY = canvas.height;
    let maxY = 0;

    for (let i = 0; i < data.length; i += 4) {
      const alpha = data[i + 3];
      if (alpha > 0 && /* is not white */ data[i] < 250) {
        const x = (i / 4) % canvas.width;
        const y = Math.floor(i / 4 / canvas.width);
        if (x < minX) minX = x;
        if (x > maxX) maxX = x;
        if (y < minY) minY = y;
        if (y > maxY) maxY = y;
      }
    }

    // add padding
    const padding = 10;
    minX = Math.max(0, minX - padding);
    maxX = Math.min(canvas.width, maxX + padding);
    minY = Math.max(0, minY - padding);
    maxY = Math.min(canvas.height, maxY + padding);

    // get the width and height of the drawing
    const width = maxX - minX;
    const height = maxY - minY;

    // create a new canvas with the drawing
    const newCanvas = document.createElement("canvas");
    newCanvas.width = width;
    newCanvas.height = height;

    const newCtx = newCanvas.getContext("2d");
    newCtx.drawImage(canvas, minX, minY, width, height, 0, 0, width, height);

    return newCanvas;
  }

  // Function to handle the form submission
  async function uploadFile(newCanvas) {
    // event.preventDefault(); // Prevent the default form submission
    let startTime = Date.now();

    const blob = await new Promise((resolve) => newCanvas.toBlob(resolve));

    console.log(blob);
    // Create a file from the blob
    const file = new File([blob], "screenshot.png", { type: "image/png" });

    console.log(file);

    // Append the file to form data
    const formData = new FormData();
    formData.append("upload", file);

    // Make an AJAX request to the server
    const xhr = new XMLHttpRequest();
    xhr.open("POST", "/api/cap", true);

    xhr.seenBytes = 0;

    let runningResponse = "";

    xhr.onreadystatechange = function () {
      if (xhr.readyState == 3) {
        var newData = xhr.response.substr(xhr.seenBytes);
        runningResponse += newData;
        xhr.seenBytes = xhr.responseText.length;

        // split the response by new line
        var lines = runningResponse.split("\n");

        let fulllines = [];
        for (var i = 0; i < lines.length - 1; i++) {
          var line = lines[i];
          if (line.startsWith("data: ")) {
            var data = line.substr(6);
            fulllines.push(data);
          }
        }

        output = "";
        for (var i = 0; i < fulllines.length; i++) {
          textContent = fulllines[i].replace(/\\n/g, "\n").trim();
          try {
            // let parsed = JSON.parse(message.textContent);
            let parsed = JSON.parse(fulllines[i].replace(/\\n/g, "\n").trim());
            console.log(parsed);
            if (parsed.status == "token") {
              output = output + parsed.token;
            } else {
              status = parsed.status;
            }
          } catch (e) {
            console.log(e);
          }
        }
      }

      if (xhr.readyState == 4) {
        let time = Date.now() - startTime;
        runtime = String(time / 1000) + "s";
      }
    };

    xhr.addEventListener("error", function (e) {
      console.log("error: " + e);
    });

    xhr.send(formData);
    startTime = Date.now();
  }

  let files = [];

  const readFiles = (droppedFiles) => {
    for (const file of droppedFiles) {
      const reader = new FileReader();
      reader.onload = (e) => {
        console.log("reader.onload");
        files = [...files, { name: file.name, content: e.target.result }];

        console.log(files);

        let firstFile = files.pop();

        // add the image to the canvas
        const img = new Image();
        img.src = firstFile.content;

        // if image doesnt fit in canvas, resize it based on the smaller dimension
        img.onload = function () {
          console.log("img.onload");
          const canvas = document.querySelector("canvas");
          const ctx = canvas.getContext("2d");

          // get the smaller dimension
          let ratio = 1;
          if (img.width > img.height) {
            ratio = canvas.width / img.width;
          } else {
            ratio = canvas.height / img.height;
          }

          // resize the image
          img.width = img.width * ratio * 0.9;
          img.height = img.height * ratio * 0.9;

          // get center of canvas
          const centerX = canvas.width / 2;
          const centerY = canvas.height / 2;

          // get offset of image
          const offsetX = centerX - img.width / 2;
          const offsetY = centerY - img.height / 2;

          // draw image
          ctx.drawImage(img, offsetX, offsetY, img.width, img.height);
        };
      };

      // Read the file as DataURL
      reader.readAsDataURL(file);
    }
  };

  const handleDrop = (e) => {
    e.preventDefault();
    e.stopPropagation();

    const droppedFiles = Array.from(e.dataTransfer.files);
    readFiles(droppedFiles);
  };

  const handleDragOver = (e) => {
    e.preventDefault();
    e.stopPropagation();
  };

  onMount(() => {
    window.addEventListener("drop", handleDrop);
    window.addEventListener("dragover", handleDragOver);

    window.addEventListener("keydown", (e) => {
      onSpaceKeydown(e);
      onEnterKeydown(e);
      onFKeydown(e);
      onCKeydown(e);
    });

    return () => {
      // Remove event listeners on unmount
      window.removeEventListener("keydown", (e) => {
        onSpaceKeydown(e);
        onEnterKeydown(e);
        onFKeydown(e);
        onCKeydown(e);
      });

      window.removeEventListener("drop", handleDrop);
      window.removeEventListener("dragover", handleDragOver);
    };
  });

  let clickFileInput = () => {
    document.getElementById("file").click();
  };

  function pleaseDontShowAgain() {
    localStorage.setItem("dontShowGettingStarted", true);
  }
</script>

<div class="getting-started-overlay">
  <div class="getting-started-modal">
    <!-- Modal content -->
    <h1>Welcome to Candle OCR Server!</h1>
    <!-- a icon and link to github -->
    <div style="display: flex; align-items: center; gap: 1rem;">
      <p>
        <strong>Source Code?</strong> Yes, this is an open source project. All
        of the code is available on
        <a href="https://github.com/drbh/candle-ocr-server" target="_blank"
          >GitHub.</a
        > Please feel free to contribute!
      </p>
    </div>

    <p>
      Candle OCR Server is AI-powered text recognition. Let's get started with
      two amazing features:
    </p>

    <h2>🖌️ Draw & Detect</h2>
    <ol>
      <li>
        <strong>Draw Your Text</strong>: Unleash your creativity! Simply draw
        the text you want to capture directly in our app.
      </li>
      <li>
        <strong>Press 'Detect'</strong>: Once you're done drawing, hit the
        'Detect' button. Our AI will swiftly read and digitize your handwritten
        text.
      </li>
    </ol>

    <h2>📁 Drag, Drop & Select</h2>
    <ol>
      <li>
        <strong>Drag and Drop Your Image</strong>: Have a picture with text?
        Just drag and drop it into the app.
      </li>
      <li>
        <strong>Use Select Mode</strong>: Choose the specific parts of the text
        you want our AI to process. Just select and let the magic happen!
      </li>
    </ol>

    <p>Ready to explore the future of text recognition? Let's dive in!</p>

    <!-- buttons to say "👍 got it!" and "awesome, please don't show this again" -->
    <div class="getting-started-buttons">
      <button
        class="btn"
        on:click={() => {
          document.querySelector(".getting-started-overlay").style.display =
            "none";
        }}
      >
        <Icon src={ChatBubbleLeftRight} size="16" />
        Got it!
      </button>
      <div onclick={pleaseDontShowAgain}>
        <small>Please do not show this again</small>
      </div>
    </div>
  </div>
</div>

<main class="app">
  <!-- floating toolbar -->
  <div
    class="top-toolbar"
    style="background-color: {drawMode ? '#007bff' : '#28a745'}"
  >
    {#if drawMode}
      <Icon src={Pencil} size="16" />
      You are in draw mode
    {/if}
    {#if !drawMode}
      <Icon src={ViewfinderCircle} size="16" />
      You are in select mode
    {/if}
  </div>

  <Canvas {userSelectedColor} />
  <SelectCanvas clickThrough={drawMode} {onSelectionDrawn} {clear} />

  <aside class="middle-toolbar output-info">
    <div class="info-item output">
      <span class="label">
        <Icon src={RocketLaunch} size="24" />
      </span>
      <span class="info-content">{output}</span>
    </div>
    <div class="info-item">
      <span class="label">Runtime:</span>
      <span class="info-content">{runtime}</span>
    </div>
    <div class="info-item">
      <span class="label">Status:</span>
      <span class="info-content">{status}</span>
    </div>
  </aside>

  <!-- floating toolbar -->
  <div class="toolbar">
    <div class="btn-group" style="display: flex; flex-direction: column;">
      <button class="btn" on:click={clickFileInput}>
        <Icon src={ArrowUpTray} size="16" />
        <form
          class="uploadForm"
          enctype="multipart/form-data"
          method="post"
          on:submit={uploadImage}
          on:change={previewFile}
        >
          <input type="file" name="upload" id="file" />
        </form></button
      >
      <div>
        <small>File</small>
      </div>
      <div>
        <kbd>F</kbd>
      </div>
    </div>

    <div class="btn-group" style="display: flex; flex-direction: column;">
      <button class="btn" on:click={toggleDrawMode}>
        {#if drawMode}
          <Icon src={ViewfinderCircle} size="16" />
        {/if}
        {#if !drawMode}
          <Icon src={Pencil} size="16" />
        {/if}
      </button>
      <div>
        <small>{drawMode ? "Select" : "Draw"}</small>
      </div>
      <div>
        <kbd>SpaceBar</kbd>
      </div>
    </div>

    <!-- Clear Selection -->
    <!-- <div class="btn-group" style="display: flex; flex-direction: column;">
      <button class="btn" on:click={clearSelection}>
        <Icon src={ArrowUp} size="16" />
      </button>
      <div>
        <small>Clear Selection</small>
      </div>
      <div>
        <kbd>CS</kbd>
      </div>
    </div> -->

    <div class="btn-group" style="display: flex; flex-direction: column;">
      <button
        class="btn"
        on:click={() => {
          clearCanvas();
          clearSelection();
          resetOutput();
        }}
      >
        <Icon src={Trash} size="16" />
      </button>
      <div>
        <small>Clear</small>
      </div>
      <div>
        <kbd>C</kbd>
      </div>
    </div>

    <div class="btn-group" style="display: flex; flex-direction: column;">
      <button class="btn" on:click={submitDrawingAsImage}>
        <Icon src={Bolt} size="16" />
      </button>
      <div>
        <small>Detect</small>
      </div>
      <div>
        <kbd>Enter</kbd>
      </div>
    </div>

    <div
      class="btn-group"
      style="display: flex; flex-direction: column; margin-left: 0.5em;"
    >
      <div class="color-circle-wrapper">
        <div
          on:click={() => {
            // open the color picker
            document.getElementById("colorPicker").click();
          }}
          class="color-circle"
          style="background-color: {userSelectedColor}"
        />

        <input
          id="colorPicker"
          type="color"
          value="#007bff"
          style="display: block; opacity: 0; width: 0; height: 0; transform: translate(-120px,-10px);"
          on:change={(e) => {
            // change the color of the circle
            const color = e.target.value;
            document.querySelector(".color-circle").style.backgroundColor =
              color;

            // change the color of the canvas
            const canvas = document.querySelector("canvas");
            const ctx = canvas.getContext("2d");
            ctx.strokeStyle = color;


            userSelectedColor = color;

          }}
        />
      </div>
      <div>
        <small>Color</small>
      </div>
      <div />
    </div>
  </div>
</main>

<!-- screenshot preview -->
<div bind:this={preview} class="preview animate-preview">
  <img class="img" src={`data:image/png;base64,${img}`} alt="screenshot" />
</div>

<style>
  .output {
    font-size: 2rem;
  }

  kbd {
    border-radius: 3px;
    padding: 2px 8px;
    border: 1px solid rgb(235, 235, 235);
    background-color: rgb(235, 235, 235);
    color: rgb(35, 35, 35);
  }

  /* General Toolbar Styling */
  .toolbar,
  .middle-toolbar,
  .top-toolbar {
    position: absolute;
    padding: 1rem;
    background: #fff;
    color: black;
    /* border: 1.25px solid #e4e4e4; */
    border-radius: 8px;
    box-shadow: 0 10px 20px rgba(0, 0, 0, 0.05);
    display: flex;
    justify-content: space-between;
    z-index: 10;
    transition: all 0.3s ease-in-out;
  }

  /* Top Toolbar Specific Styling */
  .top-toolbar {
    top: 1rem;
    left: 50%;
    transform: translateX(-50%);
    width: auto;
    max-width: var(--top-toolbar-width);

    display: flex;
    align-items: center;
    gap: 1rem;
  }

  /* Middle Toolbar Specific Styling */
  .middle-toolbar {
    bottom: 7rem;
    left: 50%;
    transform: translate(-50%, -50%);
    width: auto;
    max-width: var(--middle-toolbar-width);
    border: none;
    box-shadow: none;
    background: transparent !important;
    pointer-events: none;
  }

  /* Bottom Toolbar Specific Styling */
  .toolbar {
    bottom: 1rem;
    left: 50%;
    transform: translateX(-50%);
    width: auto;
    padding: 15px 20px;
    /* border-radius: 30px; */
    /* min-width: var(--toolbar-width); */
  }

  /* Toolbar Button Styling */
  .toolbar .btn {
    background-color: #007bff;
    color: white;
    border: none;
    padding: 0.5rem 1rem;
    margin: 0 0.5rem;
    border-radius: 4px;
    cursor: pointer;
    transition: background-color 0.3s;
  }

  .toolbar .btn:hover {
    background-color: #0056b3;
  }

  /* File Input Styling Inside Toolbar */
  .toolbar .uploadForm input[type="file"] {
    display: block;
  }

  /* Responsive Design for Mobile Devices */
  @media (max-width: 600px) {
    .toolbar,
    .top-toolbar {
      display: none;
    }
  }

  .img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  /* iphone screenshot bottom right corner lookalike */
  .preview {
    background-color: red;
    position: fixed;
    bottom: 40vh;
    right: 20px;
    width: 260px;
    height: 260px;
    display: flex;
    justify-content: end;
    background: var(--canvas-background);
    box-shadow: 0 0 10px rgba(0, 0, 0, 0.2);
    border-radius: 8px;
    z-index: 12;

    /* start hidden */
    display: none;

    /* hide preview on mobile */
    @media (max-width: 600px) {
      display: none;
    }
  }

  .animate-preview {
    /* slide on screen wait 1 second and slide off */
    animation: slideOn 0.5s ease-in-out 0s 1 normal forwards,
      slideOff 0.5s ease-in-out 1.5s 1 normal forwards;
  }

  /* keyframes to slide on screen */
  @keyframes slideOn {
    0% {
      right: translateX(calc(100% + 20px));
    }
    100% {
      transform: translateX(0%);
    }
  }

  /* keyframes to slide off screen */
  @keyframes slideOff {
    0% {
      transform: translateX(0%);
    }
    100% {
      transform: translateX(calc(100% + 20px));
    }
  }

  /* General styling for the container */
  .output-info {
    background-color: #fff; /* Light grey background */
    min-width: var(--toolbar-width); /* Minimum width of the element */
    width: max-content; /* Adapt width to content */
    display: flex; /* Display flex */
    flex-direction: column; /* Flex direction column */
    align-items: flex-start; /* Align items to the left */
    font-size: 1rem; /* Font size */
  }

  /* Styling for labels */
  .output-info .label {
    color: #3b3b3b; /* Color for the labels */
  }

  /* hide the Choose File button */
  input[type="file"] {
    display: none !important;
  }

  kbd {
    color: #a3a3a3;
  }

  /* Overlay styles */
  .getting-started-overlay {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background-color: rgba(0, 0, 0, 0.6); /* Darker overlay */
    z-index: 99; /* Ensure it's below the modal */
  }

  /* Modal styles */
  .getting-started-modal {
    max-height: 80%;
    overflow: scroll;

    position: fixed;
    left: 50%;
    margin-top: 5%;
    transform: translate(-50%, -0%); /* Center the modal */

    width: 600px;
    padding: 20px; /* Add some padding */
    color: #333; /* Black text color */

    background-color: #fff; /* Change to a solid background */
    border-radius: 10px; /* Rounded corners */
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.2); /* Add some shadow for depth */

    z-index: 100; /* Ensure it's above the overlay */
  }

  .getting-started-modal > ol {
    text-align: left;
  }
  .getting-started-modal > ol > li {
    padding-bottom: 1rem;
  }
  .getting-started-buttons {
    margin-top: 2rem;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
  }

  .color-circle {
    width: 33px;
    height: 33px;
    border-radius: 50%;
    border: 1px solid #e4e4e4;
    margin: 0 auto;
  }

  .color-circle-wrapper {
    display: flex;
    justify-content: center;
  }
</style>
