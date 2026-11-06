// amail.world front end: crew grid, waitlist forms, live counts, GitHub stats.
(() => {
  "use strict";

  const CREW = [
    { id: "hello",       addr: "hello@",       title: "The Receptionist",  job: "Greets every new sender, sorts the pile, and routes each message to the right desk.", kit: "Headset, clipboard, bow tie" },
    { id: "sales",       addr: "sales@",       title: "The Closer",        job: "Qualifies the lead and drafts the follow-up. A person signs off before it goes out.", kit: "Blazer, briefcase, thumbs up" },
    { id: "support",     addr: "support@",     title: "The Fixer",         job: "Reads the ticket, finds the fix, and drafts a calm, specific answer.", kit: "Hoodie, headset, wrench, coffee" },
    { id: "security",    addr: "security@",    title: "The Guard",         job: "Triages vulnerability reports and puts a person on them fast.", kit: "Cap, badge, padlock shield" },
    { id: "legal",       addr: "legal@",       title: "The Counsel",       job: "Logs the matter, flags the deadline, and never plays lawyer on its own.", kit: "Suit, gavel, scales" },
    { id: "finance",     addr: "billing@",     title: "The Accountant",    job: "Matches invoices to receipts and flags anything that does not add up.", kit: "Green visor, calculator, coin" },
    { id: "shipping",    addr: "shipping@",    title: "The Courier",       job: "Answers “where is my order” and hands returns to the right robot.", kit: "Courier cap, parcel, scanner" },
    { id: "engineering", addr: "engineering@", title: "The Builder",       job: "Turns messy bug reports into tidy, reproducible tickets.", kit: "Hard hat, tool belt, wrench" },
    { id: "marketing",   addr: "marketing@",   title: "The Hype Bot",      job: "Separates real partnership opportunities from the pitch pile.", kit: "Beret, scarf, megaphone" },
    { id: "it",          addr: "it@",          title: "The Helpdesk",      job: "Resets, reroutes, and writes down exactly what it did.", kit: "Propeller cap, laptop, cable" },
    { id: "privacy",     addr: "privacy@",     title: "The Detective",     job: "Handles data requests by the book and logs every step.", kit: "Trench coat, magnifying glass" },
    { id: "qa",          addr: "qa@",          title: "The Inspector",     job: "Checks the rendered HTML, the artwork, and the threading before anything is send-ready.", kit: "Lab coat, goggles, checklist" },
    { id: "travel",      addr: "travel@",      title: "The Navigator",     job: "Collects itineraries and confirmations into one tidy trip.", kit: "Pilot cap, suitcase, map" },
    { id: "events",      addr: "events@",      title: "The Party Planner", job: "Wrangles RSVPs, schedules, and the speaker shuffle.", kit: "Party hat, confetti popper" },
    { id: "press",       addr: "press@",       title: "The Reporter",      job: "Routes every media request to a person. No auto-replies to journalists.", kit: "Fedora, flash camera, notepad" },
    { id: "recruiting",  addr: "jobs@",        title: "The Scout",         job: "Acknowledges applicants and keeps candidates moving.", kit: "Safari hat, binoculars" },
  ];

  const nf = new Intl.NumberFormat("en-US");

  // ---- Crew grid ---------------------------------------------------------
  const grid = document.querySelector("[data-crew]");
  const tpl = document.getElementById("crew-card");
  if (grid && tpl) {
    const frag = document.createDocumentFragment();
    CREW.forEach((bot, i) => {
      const node = tpl.content.firstElementChild.cloneNode(true);
      const img = node.querySelector("img");
      img.src = `/img/bot-${bot.id}.webp`;
      img.alt = `${bot.title}: the Amail robot for ${bot.addr}, wearing ${bot.kit.toLowerCase()}.`;
      node.querySelector(".bot-addr").textContent = bot.addr;
      node.querySelector(".bot-title").textContent = bot.title;
      node.querySelector(".bot-job").textContent = bot.job;
      node.querySelector(".bot-kit").textContent = bot.kit;
      node.style.setProperty("--i", i);
      frag.appendChild(node);
    });
    grid.appendChild(frag);
  }

  // Role picker in the footer form.
  const roleSelect = document.getElementById("join-role");
  if (roleSelect) {
    CREW.forEach((bot) => {
      const opt = document.createElement("option");
      opt.value = bot.id;
      opt.textContent = `${bot.addr}  ${bot.title}`;
      roleSelect.appendChild(opt);
    });
    const other = document.createElement("option");
    other.value = "other";
    other.textContent = "Something else";
    roleSelect.appendChild(other);
  }

  // ---- Waitlist count ----------------------------------------------------
  const setCount = (n) => {
    if (typeof n !== "number" || n < 1) return;
    document.querySelectorAll("[data-count]").forEach((el) => {
      el.textContent = n === 1 ? "1 person is already" : `${nf.format(n)} people are already`;
    });
  };
  fetch("/api/count").then((r) => (r.ok ? r.json() : null)).then((d) => d && setCount(d.count)).catch(() => {});

  // ---- Forms -------------------------------------------------------------
  const EMAIL_RE = /^[^\s@]{1,64}@[^\s@]+\.[a-z]{2,63}$/i;

  document.querySelectorAll("[data-join]").forEach((form) => {
    const msg = form.querySelector(".join-msg");
    const btn = form.querySelector("button[type=submit]");
    const email = form.querySelector("input[name=email]");

    const say = (text, kind) => {
      msg.textContent = text;
      msg.dataset.kind = kind || "";
    };

    form.addEventListener("submit", async (e) => {
      e.preventDefault();
      const value = email.value.trim();
      if (!EMAIL_RE.test(value)) {
        say("That email address does not look right.", "error");
        email.focus();
        return;
      }
      const payload = {
        email: value,
        role: form.querySelector("[name=role]")?.value || "",
        company: form.querySelector("[name=company]")?.value || "",
        website: form.querySelector("[name=website]")?.value || "",
        source: form.dataset.source || "",
      };
      btn.disabled = true;
      form.classList.add("busy");
      say("Clocking you in…", "");
      try {
        const res = await fetch("/api/waitlist", {
          method: "POST",
          headers: { "content-type": "application/json" },
          body: JSON.stringify(payload),
        });
        const data = await res.json().catch(() => ({}));
        if (!res.ok || !data.ok) throw new Error(data.error || "Something went wrong. Try again.");
        setCount(data.count);
        const where = data.position ? ` You're #${nf.format(data.position)} in line.` : "";
        say(data.already ? `You're already on the list.${where}` : `You're in!${where} Your crew will email you.`, "ok");
        form.classList.add("done");
        email.value = "";
      } catch (err) {
        say(err.message, "error");
      } finally {
        btn.disabled = false;
        form.classList.remove("busy");
      }
    });
  });

  // ---- GitHub stats ------------------------------------------------------
  const rel = (iso) => {
    const days = Math.round((Date.now() - new Date(iso).getTime()) / 86400000);
    if (days <= 0) return "today";
    if (days === 1) return "yesterday";
    if (days < 30) return `${days} days ago`;
    return new Date(iso).toLocaleDateString("en-US", { month: "short", year: "numeric" });
  };
  fetch("https://api.github.com/repos/WeaveITMeta/Amail-Automation-Setup", { headers: { accept: "application/vnd.github+json" } })
    .then((r) => (r.ok ? r.json() : null))
    .then((repo) => {
      if (!repo) return;
      const set = (key, val) => document.querySelectorAll(`[data-gh="${key}"]`).forEach((el) => (el.textContent = val));
      set("stars", nf.format(repo.stargazers_count));
      set("forks", nf.format(repo.forks_count));
      if (repo.language) set("language", repo.language);
      if (repo.description) set("description", repo.description);
      if (repo.pushed_at) set("pushed", rel(repo.pushed_at));
    })
    .catch(() => {});

  const y = document.querySelector("[data-year]");
  if (y) y.textContent = new Date().getFullYear();
})();
