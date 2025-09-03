let redirect = localStorage.getItem("redirect") || "/";
localStorage.removeItem("redirect");

document.getElementById("login-button").addEventListener("click", async (event) => {
    event.preventDefault();

    const response = await fetch("/api/login", {
        method: "POST",
        body: JSON.stringify({
            username: document.getElementById("username").value,
            password: document.getElementById("password").value
        }),
        headers: {
            "Content-Type": "application/json"
        }
    });

    if (response.ok) {
        const data = await response.json();
        localStorage.setItem("jwt", data.token);   

        window.location.href = redirect;
    } else {
        const errorMessage = await response.text();
        document.getElementById("error-message").innerText = errorMessage;
    }
});

document.getElementById("register-button").addEventListener("click", async (event) => {
    event.preventDefault();

    const response = await fetch("/api/register", {
        method: "POST",
        body: JSON.stringify({
            username: document.getElementById("username").value,
            password: document.getElementById("password").value
        }),
        headers: {
            "Content-Type": "application/json"
        }
    });

    if (response.ok) {
        const data = await response.json();
        localStorage.setItem("jwt", data.token);

        window.location.href = redirect;
    } else {
        const errorMessage = await response.text();
        document.getElementById("error-message").innerText = errorMessage;
    }
});

document.getElementById("continue-as-guest-button").addEventListener("click", (event) => {
    event.preventDefault();

    localStorage.setItem("jwt", "guest");
    window.location.href = redirect;
});