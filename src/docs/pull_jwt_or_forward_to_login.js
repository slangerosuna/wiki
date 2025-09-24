(function () {
  const REDIRECT_TARGET = __REDIRECT_TARGET__;

  function redirectToLogin(target) {
    const encoded = encodeURIComponent(target);
    window.location.replace(`/login/?redirect=${encoded}`);
  }

  async function fetchDocument(token) {
    return fetch(window.location.href, {
      method: "GET",
      credentials: "same-origin",
      headers: {
        "Authorization": `Bearer ${token}`,
      },
    });
  }

  async function loadDocument() {
    const token = localStorage.getItem("jwt");
    if (!token) {
      redirectToLogin(REDIRECT_TARGET);
      return;
    }

    try {
      const response = await fetchDocument(token);

      if (response.status === 401) {
        localStorage.removeItem("jwt");
        redirectToLogin(REDIRECT_TARGET);
        return;
      }

      if (!response.ok) {
        console.error("Failed to load document", response.status);
        document.body.innerHTML = "<p>Unable to load document. Please try again later.</p>";
        return;
      }

      const html = await response.text();
      document.open();
      document.write(html);
      document.close();
    } catch (error) {
      console.error("Failed to load document", error);
      redirectToLogin(REDIRECT_TARGET);
    }
  }

  document.addEventListener("DOMContentLoaded", () => {
    const placeholder = document.getElementById("docs-bootstrap");
    if (placeholder) {
      placeholder.innerHTML = "<p>Loading documentation…</p>";
    }
    loadDocument();
  });
})();
