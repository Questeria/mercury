/* Mercury site — shared animations (terminal-contrast theme).
   Defensive: every block checks its elements exist, so all pages share this file. */
(function () {
  "use strict";
  var reduced = window.matchMedia("(prefers-reduced-motion: reduce)").matches;

  /* ── 1. hero/pagehead typing sequence ──────────────────────────────
     Usage: <span id="typed-cmd" data-cmd="./mercury --about"></span>
            <span id="cmd-cursor" class="typecursor"></span>
            optional: <div id="bootout"><div>…</div>…</div>            */
  var cmdEl = document.getElementById("typed-cmd");
  if (cmdEl) {
    var cmdCur = document.getElementById("cmd-cursor");
    var bootLines = Array.prototype.slice.call(document.querySelectorAll("#bootout div"));
    var CMD = cmdEl.getAttribute("data-cmd") || "";
    var showBoot = function () {
      bootLines.forEach(function (l, j) {
        setTimeout(function () { l.classList.add("on"); }, 240 + j * 260);
      });
    };
    if (reduced) {
      cmdEl.textContent = CMD;
      if (cmdCur) cmdCur.style.display = "none";
      bootLines.forEach(function (l) { l.classList.add("on"); });
    } else {
      setTimeout(function type(i) {
        i = i || 0;
        if (i <= CMD.length) {
          cmdEl.textContent = CMD.slice(0, i);
          setTimeout(function () { type(i + 1); }, 34 + Math.random() * 46);
        } else {
          if (cmdCur) cmdCur.style.display = "none";
          showBoot();
        }
      }, 350);
    }
  }

  /* ── 2. scramble-resolve for [data-scramble] ─────────────────────── */
  var GLYPHS = "!<>-_\\/[]{}—=+*^?#________";
  function scramble(el) {
    if (reduced || el.dataset.done) return;
    el.dataset.done = "1";
    var nodes = [];
    (function collect(n) {
      Array.prototype.slice.call(n.childNodes).forEach(function (c) {
        if (c.nodeType === 3) nodes.push(c); else collect(c);
      });
    })(el);
    nodes.forEach(function (node) {
      var finalText = node.textContent;
      var frame = 0, total = 18;
      (function step() {
        frame++;
        var t = frame / total;
        node.textContent = finalText.split("").map(function (ch, i) {
          if (ch === " " || ch === " ") return ch;
          return (i / finalText.length < t) ? ch : GLYPHS[(Math.random() * GLYPHS.length) | 0];
        }).join("");
        if (frame < total) setTimeout(step, 36);
        else node.textContent = finalText;
      })();
    });
  }

  /* ── 3. reveal on scroll ──────────────────────────────────────────── */
  if ("IntersectionObserver" in window) {
    var io = new IntersectionObserver(function (entries) {
      entries.forEach(function (e) {
        if (!e.isIntersecting) return;
        e.target.classList.add("in");
        if (e.target.hasAttribute("data-scramble")) scramble(e.target);
        if (e.target.id === "verifybox") runVerify();
        io.unobserve(e.target);
      });
    }, { threshold: 0.18 });
    Array.prototype.slice.call(document.querySelectorAll(".reveal")).forEach(function (el) { io.observe(el); });
  } else {
    Array.prototype.slice.call(document.querySelectorAll(".reveal")).forEach(function (el) { el.classList.add("in"); });
  }

  /* ── 4. verify block: types the command, prints OK ────────────────── */
  var verifyDone = false;
  function runVerify() {
    if (verifyDone) return; verifyDone = true;
    var t = document.getElementById("verify-typed");
    var cur = document.getElementById("verify-cursor");
    var res = document.getElementById("verify-result");
    var note = document.getElementById("verify-note");
    if (!t) return;
    var VCMD = t.getAttribute("data-cmd") || "sha256sum -c Mercury-Linux-amd64.deb.sha256";
    if (reduced) {
      t.textContent = VCMD;
      if (cur) cur.style.display = "none";
      if (res) res.style.opacity = 1;
      if (note) note.style.opacity = 1;
      return;
    }
    var i = 0;
    (function step() {
      if (i <= VCMD.length) {
        t.textContent = VCMD.slice(0, i++);
        setTimeout(step, 26 + Math.random() * 38);
      } else {
        if (cur) cur.style.display = "none";
        setTimeout(function () {
          if (res) { res.style.transition = "opacity .25s"; res.style.opacity = 1; res.classList.add("shown"); }
          if (note) setTimeout(function () { note.style.transition = "opacity .4s"; note.style.opacity = 1; }, 600);
        }, 350);
      }
    })();
  }

  /* ── 5. copy buttons: [data-copy="text to copy"] ──────────────────── */
  Array.prototype.slice.call(document.querySelectorAll("[data-copy]")).forEach(function (btn) {
    var original = btn.textContent;
    btn.addEventListener("click", function () {
      var text = btn.getAttribute("data-copy");
      function done() {
        btn.classList.add("copied"); btn.textContent = "copied ✓";
        setTimeout(function () { btn.classList.remove("copied"); btn.textContent = original; }, 1600);
      }
      function fallback() {
        var ta = document.createElement("textarea");
        ta.value = text; document.body.appendChild(ta); ta.select();
        try { document.execCommand("copy"); done(); } catch (e) {}
        document.body.removeChild(ta);
      }
      if (navigator.clipboard && navigator.clipboard.writeText) {
        navigator.clipboard.writeText(text).then(done, fallback);
      } else { fallback(); }
    });
  });
})();
