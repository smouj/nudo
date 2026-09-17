// The book's client-side behaviour. Small on purpose: this is a manual, not an
// application, and every line here is code a reader has to download.
//
//   1. tags the document as the engineering-paper edition, for the stylesheet;
//   2. labels every code block with its source, an editorial convention of the
//      book rather than a feature of mdBook;
//   3. builds the language switcher.
//
// The switcher is the only part with real logic, and it is written to fail
// quietly: if the path cannot be understood, no control is shown and the page is
// unaffected. A navigation aid that breaks navigation is worse than no aid.

(() => {
  "use strict";

  // Languages the site is built in, and how they are named to a reader.
  // Kept in step with book/scripts/build_site.py; a language added there but not
  // here simply does not appear in the switcher.
  const LANGUAGES = {
    en: "English",
    es: "Español",
    zh: "简体中文",
    ja: "日本語",
    de: "Deutsch",
  };
  const CANONICAL = "en";

  document.documentElement.dataset.nudoBook = "engineering-paper";

  for (const pre of document.querySelectorAll("pre")) {
    pre.setAttribute("data-doc", "NUDO-SRC");
  }

  /**
   * Works out where the site root and the current chapter are, so that the same
   * chapter can be addressed in another language.
   *
   * Layout the function depends on, produced by book/scripts/build_site.py:
   *
   *   /<site>/                     English, at the root
   *   /<site>/<lang>/              every other language
   *   /<site>/[<lang>/]<NN-section>/<page>.html
   *
   * Chapters are recognised by their `NN-` prefix, which the book's own numbering
   * already guarantees, so the site root does not have to be configured or
   * guessed from the domain.
   */
  function locate(pathname) {
    const segments = pathname.split("/").filter((part) => part.length > 0);
    const page = segments.length > 0 ? segments.pop() : "index.html";
    const rest = segments.slice();

    const firstChapter = rest.findIndex((part) => /^\d{2}-/.test(part));
    const site = firstChapter === -1 ? rest.slice() : rest.slice(0, firstChapter);
    const chapter = firstChapter === -1 ? [] : rest.slice(firstChapter);

    let current = CANONICAL;
    const last = site.length > 0 ? site[site.length - 1] : null;
    if (last !== null && Object.prototype.hasOwnProperty.call(LANGUAGES, last)) {
      current = last;
      site.pop();
    }

    return { site, chapter, page, current };
  }

  function addressFor(target, place) {
    const parts = place.site.concat(target === CANONICAL ? [] : [target], place.chapter, [place.page]);
    return "/" + parts.filter((part) => part.length > 0).join("/");
  }

  /**
   * The languages the site was actually built with, from the manifest that
   * build_site.py writes. Offering a language that does not exist yet would send
   * a reader to a 404, which is worse than not offering it. Falls back to the
   * full list if the manifest cannot be read, so a copy of the site served from
   * a file system still gets a switcher.
   */
  async function availableLanguages(place) {
    try {
      const url = "/" + place.site.concat(["languages.json"]).filter((p) => p.length > 0).join("/");
      const response = await fetch(url, { cache: "no-cache" });
      if (!response.ok) {
        return Object.keys(LANGUAGES);
      }
      const manifest = await response.json();
      const listed = Array.isArray(manifest.languages) ? manifest.languages : [];
      const known = listed.filter((code) => Object.prototype.hasOwnProperty.call(LANGUAGES, code));
      return known.length > 0 ? known : Object.keys(LANGUAGES);
    } catch (error) {
      return Object.keys(LANGUAGES);
    }
  }

  async function buildSwitcher() {
    const place = locate(window.location.pathname);
    if (!Object.prototype.hasOwnProperty.call(LANGUAGES, place.current)) {
      return;
    }
    const offered = await availableLanguages(place);
    if (offered.length < 2) {
      return;
    }

    const host = document.querySelector("#menu-bar .right-buttons") || document.querySelector("#menu-bar");
    if (host === null) {
      return;
    }

    const wrapper = document.createElement("div");
    wrapper.className = "nudo-languages";

    const label = document.createElement("label");
    label.className = "nudo-languages__label";
    label.setAttribute("for", "nudo-language-select");
    label.textContent = "Language";

    const select = document.createElement("select");
    select.id = "nudo-language-select";
    select.className = "nudo-languages__select";

    for (const code of offered) {
      const option = document.createElement("option");
      option.value = code;
      option.textContent = LANGUAGES[code];
      option.selected = code === place.current;
      select.append(option);
    }

    select.addEventListener("change", () => {
      const target = select.value;
      if (target === place.current) {
        return;
      }
      window.location.assign(addressFor(target, place));
    });

    wrapper.append(label, select);
    host.prepend(wrapper);
  }

  try {
    buildSwitcher();
  } catch (error) {
    // A manual must never be broken by its own navigation aid.
    console.warn("language switcher unavailable:", error);
  }
})();
