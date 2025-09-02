let jwt = localStorage.getItem("jwt");
if (!jwt) {
    localStorage.setItem("redirect_after_login", window.location.href);
    window.location.href = "/login";
} else {
    fetch(window.location.href, {
        headers: {
            "Authorization": `Bearer ${jwt}`
        }
    });
}