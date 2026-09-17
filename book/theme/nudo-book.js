(() => {
  document.documentElement.dataset.nudoBook = "engineering-paper";
  for (const pre of document.querySelectorAll("pre")) {
    pre.setAttribute("data-doc", "NUDO-SRC");
  }
})();
