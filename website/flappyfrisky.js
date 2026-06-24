// watermelon cat

const squeaky = new Audio("./media/squeakytoy.ogg");
function squeak() {
  const src = squeaky.cloneNode();
  src.preservesPitch = false;
  src.playbackRate = 1.3 ** Math.random();
  src.volume = 0.05;
  src.play();
}

const friskyImage = document.getElementById("frisky");

let numHotDogClicks = 0;
friskyImage.addEventListener("click", () => {
  friskyImage.width = Math.random() * (750 - 20) + 20;
  friskyImage.height = Math.random() * (499 - 20) + 20;
  squeak();
  if (++numHotDogClicks == 5) {
    loadFlappyFrisky();
    flappyfrisky.startGame();
    numHotDogClicks = 0;
  }
});

function loadFlappyFrisky() {
  if (window.flappyfrisky) return;
  const exports = (window.flappyfrisky = {});

  /** @returns { HTMLElement } */
  const $ = (sel, el) => (el ?? document).querySelector(sel);
  /** @returns { HTMLElement } */
  const classy = (tag, c) => {
    const el = document.createElement(tag);
    if (c) el.className = c;
    return el;
  };

  const frisky = classy("img");
  frisky.src = "./media/frisky.png";

  const explode = new Audio("./media/explode_instant.ogg");
  const barSound = new Audio("./media/bar.ogg");
  const maow = new Audio("./media/cypher-maow.ogg");

  const explodeGif = classy("img", "ff-explode");
  explodeGif.src = "./media/explosion_loop.gif";

  let isDead;
  const deadEl = classy("div", "ff-dead");
  deadEl.innerHTML = `
<div>
  <p class="ff-title">frisky fcuing died :(</p>
  <p class="ff-score"></p>
  <button onclick="setTimeout(flappyfrisky.restartGame)">Restart game</button> <button onclick="flappyfrisky.endGame()">Quit</button>
  <p class="ff-credits">
    frisky oc by glydedagamer
  </p>
  <p class="ff-credits">
    coding by <span id="ff-cypher" onclick="flappyfrisky.maow()">cyexclam</span>
  </p>
</div>
`;
  const scoreEl = $(".ff-score", deadEl);
  /** @type { HTMLCanvasElement } */
  const canvasEl = classy("canvas");
  const containerEl = classy("div", "ff-container");
  containerEl.append(canvasEl, deadEl);

  document.body.addEventListener("keydown", flap);
  document.body.addEventListener("mousedown", flap);
  document.body.addEventListener("touchstart", flap, { passive: false });

  let timeSinceFlap = 0;
  function flap(e) {
    if (
      e.type === "keydown" &&
      (e.ctrlKey || e.shiftKey || e.altKey || e.metaKey)
    )
      return;
    if (animationFrame === undefined || isDead) return;
    if (e.type === "touchstart" && !e.touches) return;
    e.preventDefault();
    e.stopImmediatePropagation();

    timeSinceFlap = 1;
    pvel = -1000;
    squeak();
  }

  const ctx = canvasEl.getContext("2d");

  const link = classy("link");
  link.href = "./flappyfrisky.css";
  link.rel = "stylesheet";
  document.head.appendChild(link);

  let animationFrame;
  let bgColor, fgColor;
  let width, height;
  function onResize() {
    canvasEl.width = width = innerWidth;
    canvasEl.height = height = innerHeight;
  }
  addEventListener("resize", onResize);
  exports.startGame = function () {
    animationFrame = requestAnimationFrame(draw);
    isDead = false;

    onResize();
    const bec = $("#beepboxEditorContainer");
    bec.scrollIntoView({ behavior: "smooth" });
    const computed = getComputedStyle(bec);
    bgColor = computed.getPropertyValue("--ui-widget-background");
    fgColor = computed.getPropertyValue("--primary-text");
    document.body.append(containerEl);
    document.head.appendChild(link);
    containerEl.focus();

    bgTransparency = 1;

    exports.restartGame();
  };
  exports.endGame = function () {
    cancelAnimationFrame(animationFrame);
    animationFrame = startTime = curTime = prevTime = dt = undefined;

    containerEl.remove();
  };
  exports.restartGame = function () {
    isDead = false;
    pposy = height / 2;
    pvel = 0;
    barVel = 400;
    barSpace = 300;
    score = 0;
    bars.length = 0;
    nextBarDist = Math.max(600, width / 3);
    startTime = undefined;
  };
  exports.maow = function () {
    const src = maow.cloneNode();
    src.preservesPitch = false;
    src.playbackRate = 1.3 ** (Math.random() - 0.5);
    src.volume = 0.1;
    src.play();
  };

  let bgTransparency;
  let startTime, curTime, prevTime, dt;
  let pposx, pposy, pvel;
  let barVel;
  let barSpace, nextBarDist;
  let score;
  let successTimer, successBar;
  const BAR_WIDTH = 100;

  function draw(t) {
    animationFrame = requestAnimationFrame(draw);

    curTime = t;
    startTime ??= curTime;
    dt = (curTime - (prevTime ?? curTime)) / 1000;
    prevTime = curTime;

    pposy += pvel * dt;
    pvel += 3000 * dt;
    pposx = width * 0.1;

    ctx.resetTransform();
    ctx.clearRect(0, 0, width, height);
    bgTransparency = Math.max(0, bgTransparency - dt * 2);
    ctx.globalCompositeOperation = "source-over";
    ctx.globalAlpha = 1 - bgTransparency;
    ctx.fillStyle = bgColor;
    ctx.fillRect(0, 0, width, height);
    ctx.globalAlpha = 1;

    // bars
    ctx.globalCompositeOperation = "destination-out";
    let barsDeleted = 0;
    for (const bar of bars) {
      bar.x -= barVel * dt;
      const { x, ytop, ybottom } = bar;
      if (bar.x + BAR_WIDTH < 0) {
        barsDeleted++;
        continue;
      }

      ctx.fillRect(x, 0, BAR_WIDTH, ytop);
      ctx.fillRect(x, ybottom, BAR_WIDTH, height - ybottom);

      if (
        pposx > x &&
        pposx < x + BAR_WIDTH &&
        (pposy < ytop || pposy > ybottom)
      )
        die();
      if (!isDead && pposx > x + BAR_WIDTH && !bar.marked) {
        bar.marked = true;
        barSound.volume = 0.2;
        barSound.preservesPitch = false;
        barSound.playbackRate = 1 + (curTime - startTime) / 1e5;
        barSound.play();
        score++;
        successTimer = 1;
        successBar = bar;
      }
    }

    bars.splice(0, barsDeleted);

    ctx.globalCompositeOperation = "source-over";
    if (successTimer > 0.01) {
      successTimer *= 0.001 ** dt;
      ctx.fillStyle = fgColor;
      ctx.globalAlpha = successTimer * 0.1;
      ctx.fillRect(successBar.x, 0, BAR_WIDTH, height);
      ctx.globalAlpha = 1;
    }

    nextBarDist -= barVel * dt;
    while (nextBarDist < width + BAR_WIDTH) {
      makeBar(nextBarDist);
      nextBarDist += 500;
    }

    // friskaayyy!!!
    if (!isDead) {
      ctx.resetTransform();
      ctx.translate(pposx, pposy);
      ctx.scale(-0.2, 0.2);
      const velFac = 1.05 + (pvel * 0.0004) ** 2 - timeSinceFlap * 0.5;
      timeSinceFlap *= 0.002 ** dt;
      ctx.scale(1 / velFac, velFac);
      ctx.rotate(Math.atan2(-pvel, barVel * 2));
      ctx.drawImage(frisky, -611 / 2, -595 / 2);

      ctx.resetTransform();
      ctx.font = `20px "Comic Sans MS"`;
      ctx.fillStyle = fgColor;
      ctx.fillText("Score: " + score, 20, 30);
    }

    if (pposy < -50 || pposy > height + 50) die();

    if (!isDead) {
      barSpace *= 0.5 ** (dt / 20);
      barVel *= 2 ** (dt / 40);
    }

    deadEl.style.display = isDead ? "" : "none";
  }

  let numTimesDied = 0;
  function die() {
    if (isDead) return;
    isDead = true;

    const cur = explode.cloneNode();
    cur.volume = 0.1;
    cur.play();

    scoreEl.textContent = "Score: " + score;
    explodeGif.style.left = pposx + "px";
    explodeGif.style.top = Math.min(height - 50, Math.max(50, pposy)) + "px";
    setTimeout(() => explodeGif.remove(), 1400);
    containerEl.append(explodeGif);

    const titleEl = $(".ff-title", deadEl);
    numTimesDied++;

    // everything's encoded... i guess you could call it a   cypher
    const FUNNY_MESSAGES = {
      10: "ũťŧůĪťŤĦĪųťſĪũūŤĪŮťĪŨůžžůŸĪžŢūŤĪžŢūžĤ",
      20: "ŰŻźĳŠĴŭŻšĴżŵŢűĴŵźŭŠżŽźųĴŶűŠŠűŦĴŠŻĴŰŻī",
      30: "ũŶŻŰľŪŶŻŧľŭſŷźľŨŷźŻűľŹſųŻŭľũŻŬŻľſźźŷŽŪŷŨŻĲľŷĹųľŭūŬŻľŪŶŻŧľźŷźŰĹŪľųŻſŰľŪŶŷŭľűŰŻİ",
      40: "ĊŁĈŋŉņĈśŜŇŘĈŉŜĈŉņőĈŜŁŅōĊĈąĈőŇŝĄĈŘŚŇŊŉŊńő",
      50: "şœŝŅĒşŀŀłĒłŇŀŀĒşŀœœŝŝŇĒĈāő",
      51: "ŀŜŁŁŊēŔņŊŀēńŁŜŝŔēŐśŒŝŝŖş",
      60: "ŘŕŏŌřŎŏřŎĜĀď",
      67: "ĺĬĶŤĵĦţĨĪįįĦħŭŭŭţħĬĭŤķţİĢĺţĪķŭ",
      69: "ļĪİŢĳĠťĮĬĩĩĠġťųżťģķĬĶĮĬĠĶūťģĬīĠũťıĭĬĶťĢĠıĶťĤťĵĤĶĶū",
      80: "ĸĹŰĸĹĦĵ",
      81: "",
      [Infinity]: "ţŨťšŴťŲĮ",
    };

    if (FUNNY_MESSAGES[numTimesDied])
      titleEl.innerHTML = String.fromCharCode(
        ...[...FUNNY_MESSAGES[numTimesDied]].map(
          (x) => (x.charCodeAt() ^ numTimesDied) & 0xff,
        ),
      );
    else if (numTimesDied > 25 && numTimesDied < 80)
      titleEl.textContent = `you've killed ${numTimesDied} friskies. i hope you're happy with yourself.`;
  }

  const bars = [];

  function makeBar(x) {
    const height = (Math.random() * 0.6 + 0.2) * innerHeight;
    bars.push({
      x,
      ytop: height - barSpace,
      ybottom: height + barSpace,
    });
  }
}
