// --- Foldable Account Settings ---
document.addEventListener('DOMContentLoaded', function() {
    const foldable = document.querySelector('.foldable-settings-container');
    if (foldable) {
        foldable.addEventListener('mouseenter', () => {
            foldable.classList.add('unfolded');
        });
        foldable.addEventListener('mouseleave', () => {
            foldable.classList.remove('unfolded');
        });
    }
});
// --- 2FA Toggle Logic ---
const accountSettingsSection = document.getElementById("account-settings-section");
const settingsForm = document.getElementById("settings-form");
const twoFAToggle = document.getElementById("2fa-toggle");
const settingsErrAlert = document.getElementById("settings-err-alert");
const twoFAStatusBadge = document.getElementById("2fa-status-badge");
const twoFAMethodRow = document.getElementById("2fa-method-row");
const twoFAMethodSelect = document.getElementById("2fa-method-select");
const twoFAMethodSetup = document.getElementById("2fa-method-setup");

let current2FAStatus = false;
let current2FAMethod = "Email";

function showAccountSettings(currentRequires2FA, method) {
    accountSettingsSection.style.display = "block";
    twoFAToggle.checked = !!currentRequires2FA;
    current2FAStatus = !!currentRequires2FA;
    current2FAMethod = method || "Email";
    update2FAStatusBadge();
    twoFAMethodRow.style.display = current2FAStatus ? "block" : "none";
    twoFAMethodSelect.value = current2FAMethod;
    show2FAMethodSetup(current2FAMethod);
}
function hideAccountSettings() {
    accountSettingsSection.style.display = "none";
}
function update2FAStatusBadge() {
    if (!twoFAStatusBadge) return;
    if (current2FAStatus) {
        twoFAStatusBadge.className = "badge bg-success ms-2";
        twoFAStatusBadge.textContent = `2FA: Enabled (${current2FAMethod})`;
    } else {
        twoFAStatusBadge.className = "badge bg-secondary ms-2";
        twoFAStatusBadge.textContent = "2FA: Disabled";
    }
}
function show2FAMethodSetup(method) {
    if (!twoFAMethodSetup) return;
    if (!current2FAStatus) {
        twoFAMethodSetup.style.display = "none";
        twoFAMethodSetup.innerHTML = "";
        return;
    }
    twoFAMethodSetup.style.display = "block";
    if (method === "AuthenticatorApp") {
        twoFAMethodSetup.innerHTML = `<div class='alert alert-info'>Scan the QR code with your authenticator app. <br><span class='text-muted'>(Stub QR code here)</span></div>`;
    } else if (method === "SMS") {
        twoFAMethodSetup.innerHTML = `<div class='alert alert-info'>Enter your phone number to receive codes via SMS. <br><span class='text-muted'>(Stub phone input here)</span></div>`;
    } else {
        twoFAMethodSetup.innerHTML = "";
        twoFAMethodSetup.style.display = "none";
    }
}

twoFAToggle.addEventListener("change", async (e) => {
    e.preventDefault();
    const enable = twoFAToggle.checked;
    const action = enable ? "enable" : "disable";
    if (!confirm(`Are you sure you want to ${action} 2FA?`)) {
        twoFAToggle.checked = !enable;
        return;
    }
    try {
        const res = await fetch('/account/settings', {
            method: 'PATCH',
            headers: {
                'Content-Type': 'application/json',
                ...(currentUserEmail ? { 'x-user-email': currentUserEmail } : {})
            },
            body: JSON.stringify({ requires2FA: enable, twoFAMethod: twoFAMethodSelect.value })
        });
        if (res.ok) {
            settingsErrAlert.style.display = "none";
            current2FAStatus = enable;
            update2FAStatusBadge();
            twoFAMethodRow.style.display = enable ? "block" : "none";
            // --- Email notification stub ---
            console.log(`[STUB] Email notification: 2FA has been ${enable ? "enabled" : "disabled"}.`);
            alert(`2FA has been ${enable ? "enabled" : "disabled"}.`);
        } else {
            const data = await res.json();
            let msg = data && data.error ? data.error : 'Failed to update 2FA setting.';
            settingsErrAlert.innerHTML = `<span><strong>Error: </strong>${msg}</span>`;
            settingsErrAlert.style.display = "block";
            twoFAToggle.checked = !enable;
        }
    } catch (err) {
        settingsErrAlert.innerHTML = `<span><strong>Error: </strong>Network error</span>`;
        settingsErrAlert.style.display = "block";
        twoFAToggle.checked = !enable;
    }
});

twoFAMethodSelect.addEventListener("change", async (e) => {
    const newMethod = twoFAMethodSelect.value;
    if (!current2FAStatus) return;
    if (!confirm(`Change 2FA method to ${newMethod}?`)) {
        twoFAMethodSelect.value = current2FAMethod;
        return;
    }
    try {
        const res = await fetch('/account/settings', {
            method: 'PATCH',
            headers: {
                'Content-Type': 'application/json',
                ...(currentUserEmail ? { 'x-user-email': currentUserEmail } : {})
            },
            body: JSON.stringify({ requires2FA: true, twoFAMethod: newMethod })
        });
        if (res.ok) {
            settingsErrAlert.style.display = "none";
            current2FAMethod = newMethod;
            update2FAStatusBadge();
            show2FAMethodSetup(newMethod);
            // --- Email notification stub ---
            console.log(`[STUB] Email notification: 2FA method changed to ${newMethod}.`);
            alert(`2FA method changed to ${newMethod}.`);
        } else {
            const data = await res.json();
            let msg = data && data.error ? data.error : 'Failed to update 2FA method.';
            settingsErrAlert.innerHTML = `<span><strong>Error: </strong>${msg}</span>`;
            settingsErrAlert.style.display = "block";
            twoFAMethodSelect.value = current2FAMethod;
        }
    } catch (err) {
        settingsErrAlert.innerHTML = `<span><strong>Error: </strong>Network error</span>`;
        settingsErrAlert.style.display = "block";
        twoFAMethodSelect.value = current2FAMethod;
    }
});
// --- Delete Account Button Logic ---
const deleteAccountBtn = document.getElementById("delete-account-btn");
let currentUserEmail = null; // Track logged-in user

// Show delete button only when logged in
function showDeleteButton(email) {
    currentUserEmail = email;
    deleteAccountBtn.style.display = "inline-block";
    // Fetch current 2FA settings from backend
    fetch('/account/settings', {
        method: 'GET',
        headers: {
            'Content-Type': 'application/json',
            ...(currentUserEmail ? { 'x-user-email': currentUserEmail } : {})
        }
    })
    .then(res => res.ok ? res.json() : Promise.reject(res))
    .then(data => {
        showAccountSettings(data.requires2FA, data.twoFAMethod);
    })
    .catch(() => {
        showAccountSettings(false, "Email");
    });
}
function hideDeleteButton() {
    currentUserEmail = null;
    deleteAccountBtn.style.display = "none";
}

deleteAccountBtn.addEventListener("click", async () => {
    if (!currentUserEmail) {
        alert("No user logged in.");
        return;
    }
    if (!confirm("Are you sure you want to delete your account? This cannot be undone.")) return;
    deleteAccountBtn.disabled = true;
    const res = await fetch('/delete-account', {
        method: 'DELETE',
        headers: {
            'Content-Type': 'application/json',
            'x-user-email': currentUserEmail
        }
    });
    deleteAccountBtn.disabled = false;
    if (res.ok) {
        alert('Account deleted.');
        hideDeleteButton();
        // Log out, clear forms, etc.
        loginSection.style.display = "block";
        twoFASection.style.display = "none";
        signupSection.style.display = "none";
    hideAccountSettings();
        loginForm.email.value = "";
        loginForm.password.value = "";
        signupForm.email.value = "";
        signupForm.password.value = "";
        signupForm.twoFA.checked = false;
    } else {
        let msg = 'Failed to delete account.';
        try {
            const data = await res.json();
            if (data && data.error) msg = data.error;
        } catch {}
        alert(msg);
    }
});
const loginSection = document.getElementById("login-section");
const twoFASection = document.getElementById("2fa-section");
const signupSection = document.getElementById("signup-section");

const signupLink = document.getElementById("signup-link");
const twoFALoginLink = document.getElementById("2fa-login-link");
const signupLoginLink = document.getElementById("signup-login-link");

signupLink.addEventListener("click", (e) => {
    e.preventDefault();

    loginSection.style.display = "none";
    twoFASection.style.display = "none";
    signupSection.style.display = "block";
});

twoFALoginLink.addEventListener("click", (e) => {
    e.preventDefault();

    loginSection.style.display = "block";
    twoFASection.style.display = "none";
    signupSection.style.display = "none";
});

signupLoginLink.addEventListener("click", (e) => {
    e.preventDefault();

    loginSection.style.display = "block";
    twoFASection.style.display = "none";
    signupSection.style.display = "none";
});

// -----------------------------------------------------

const loginForm = document.getElementById("login-form");
const loginButton = document.getElementById("login-form-submit");
const loginErrAlter = document.getElementById("login-err-alert");

loginButton.addEventListener("click", async (e) => {
    e.preventDefault();

    const email = loginForm.email.value;
    const password = loginForm.password.value;


    fetch('/login', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
    body: JSON.stringify({ email, password }),
    }).then(response => {
        if (response.status === 206) {
            TwoFAForm.email.value = email;
            response.json().then(data => {
                TwoFAForm.login_attempt_id.value = data.loginAttemptId;
            });

            loginForm.email.value = "";
            loginForm.password.value = "";

            loginSection.style.display = "none";
            twoFASection.style.display = "block";
            signupSection.style.display = "none";
            loginErrAlter.style.display = "none";
        } else if (response.status === 200) {
            loginForm.email.value = "";
            loginForm.password.value = "";
            loginErrAlter.style.display = "none";
            // Show delete button for logged-in user
            showDeleteButton(email);
            alert("You have successfully logged in.");
        } else {
            response.json().then(data => {
                let error_msg = data.error;
                if (error_msg !== undefined && error_msg !== null && error_msg !== "") {
                    loginErrAlter.innerHTML = `<span><strong>Error: </strong>${error_msg}</span>`;
                    loginErrAlter.style.display = "block";
                } else {
                    loginErrAlter.style.display = "none";
                }
            });
        }
    });
});

const signupForm = document.getElementById("signup-form");
const signupButton = document.getElementById("signup-form-submit");
const signupErrAlter = document.getElementById("signup-err-alert");

signupButton.addEventListener("click", async (e) => {
    e.preventDefault();

    const email = signupForm.email.value;
    const password = signupForm.password.value;
    const requires2FA = signupForm.twoFA.checked;


    fetch('/signup', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
    body: JSON.stringify({ email, password, requires2FA }),
    }).then(response => {
        if (response.ok) {
            signupForm.email.value = "";
            signupForm.password.value = "";
            signupForm.twoFA.checked = false;
            signupErrAlter.style.display = "none";
            alert("You have successfully created a user.");
            loginSection.style.display = "block";
            twoFASection.style.display = "none";
            signupSection.style.display = "none";
            // Optionally, show delete button for new user (if auto-login)
            // showDeleteButton(email);
        } else {
            response.json().then(data => {
                let error_msg = data.error;
                if (error_msg !== undefined && error_msg !== null && error_msg !== "") {
                    signupErrAlter.innerHTML = `<span><strong>Error: </strong>${error_msg}</span>`;
                    signupErrAlter.style.display = "block";
                } else {
                    signupErrAlter.style.display = "none";
                }
            });
        }
    });
});

const TwoFAForm = document.getElementById("2fa-form");
const TwoFAButton = document.getElementById("2fa-form-submit");
const TwoFAErrAlter = document.getElementById("2fa-err-alert");

TwoFAButton.addEventListener("click", (e) => {
    e.preventDefault();

    const email = TwoFAForm.email.value;
    const loginAttemptId = TwoFAForm.login_attempt_id.value;
    const TwoFACode = TwoFAForm.email_code.value;

    fetch('/verify-2fa', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify({ email, loginAttemptId, "2FACode": TwoFACode }),
    }).then(response => {
        if (response.ok) {
            TwoFAForm.email.value = "";
            TwoFAForm.email_code.value = "";
            TwoFAForm.login_attempt_id.value = "";
            TwoFAErrAlter.style.display = "none";
            alert("You have successfully logged in.");
            loginSection.style.display = "block";
            twoFASection.style.display = "none";
            signupSection.style.display = "none";
            showDeleteButton(email);
        } else {
            response.json().then(data => {
                let error_msg = data.error;
                if (error_msg !== undefined && error_msg !== null && error_msg !== "") {
                    TwoFAErrAlter.innerHTML = `<span><strong>Error: </strong>${error_msg}</span>`;
                    TwoFAErrAlter.style.display = "block";
                } else {
                    TwoFAErrAlter.style.display = "none";
                }
            });
        }
    });
});