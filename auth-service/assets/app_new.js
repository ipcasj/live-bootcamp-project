(function(){
  const VIEWS = {LOGIN: 'login-section', SIGNUP: 'signup-section', TWO_FA: '2fa-section', FORGOT: 'forgot-password-section', SETTINGS: 'account-settings-section'};
  const els = {};
  const st = {view: VIEWS.LOGIN, user: null, twoFA: {enabled: false, method: 'Email'}};

  function $(id) { return document.getElementById(id); }
  function hide(el) { if (el) el.style.display = 'none'; }
  function show(el) { if (el) el.style.display = 'block'; }
  function clear(el) { if (el) { el.style.display = 'none'; el.innerHTML = ''; } }
  function err(el, msg) { if (el) { el.innerHTML = `<strong>Error:</strong> ${escapeHtml(msg)}`; el.style.display = 'block'; } }
  function escapeHtml(s) { return s.replace(/[&<>"']/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;','\'':'&#39;'}[c])); }

  function focusFirstInvalid(inputs) {
    for (const inp of inputs) {
      if (!inp) continue;
      if (inp.value === '' || inp.getAttribute('aria-invalid') === 'true') {
        try { inp.focus(); } catch {}
        return;
      }
    }
  }

  function showToast(title, body, variant = 'success', delay = 3500) {
    const container = $('toast-container');
    if (!container) { alert(body); return; }
    const id = 't_' + Math.random().toString(36).slice(2);
    const markup = `<div id="${id}" class="toast align-items-center text-bg-${variant} border-0 mb-2" role="alert" aria-live="assertive" aria-atomic="true" data-bs-delay="${delay}">
      <div class="d-flex">
        <div class="toast-body"><strong>${escapeHtml(title)}:</strong> ${escapeHtml(body)}</div>
        <button type="button" class="btn-close btn-close-white me-2 m-auto" data-bs-dismiss="toast" aria-label="Close"></button>
      </div>
    </div>`;
    container.insertAdjacentHTML('beforeend', markup);
    const el = container.lastElementChild;
    if (window.bootstrap && bootstrap.Toast) {
      const t = new bootstrap.Toast(el);
      t.show();
      el.addEventListener('hidden.bs.toast', () => el.remove());
    } else {
      el.classList.add('show');
      setTimeout(() => el.remove(), delay + 500);
    }
  }

  async function api(path, opt = {}) {
    const o = { headers: {'Content-Type': 'application/json', ...opt.headers}, ...opt };
    if (o.body && typeof o.body !== 'string') o.body = JSON.stringify(o.body);
    let r, d;
    try { r = await fetch(path, o); } catch (e) { throw { message: 'Network error', network: true }; }
    try { d = await r.json(); } catch { d = {}; }
    if (!r.ok) throw { message: d.error || d.message || 'Request failed', status: r.status, data: d };
    return d;
  }

  function swap(v) {
    st.view = v;
    Object.values(VIEWS).forEach(id => hide($(id)));
    show($(v));
  }

  function loggedIn(email) {
    st.user = email;
    show($(VIEWS.SETTINGS));
    update2FABadge();
  }

  function loggedOut() {
    st.user = null;
    hide($(VIEWS.SETTINGS));
  }

  function update2FABadge() {
    const b = $('2fa-status-badge');
    if (!b) return;
    if (st.twoFA.enabled) {
      b.className = 'badge bg-success ms-2';
      b.textContent = `2FA: Enabled (${st.twoFA.method})`;
      show($('2fa-method-row'));
    } else {
      b.className = 'badge bg-secondary ms-2';
      b.textContent = '2FA: Disabled';
      hide($('2fa-method-row'));
    }
    show2FAMethodSetup();
  }

  function show2FAMethodSetup() {
    const c = $('2fa-method-setup');
    if (!c) return;
    if (!st.twoFA.enabled) {
      c.style.display = 'none';
      c.innerHTML = '';
      return;
    }
    c.style.display = 'block';
    if (st.twoFA.method === 'AuthenticatorApp') {
      c.innerHTML = "<div class='alert alert-info py-1 px-2'>Scan QR (stub)</div>";
    } else if (st.twoFA.method === 'SMS') {
      c.innerHTML = "<div class='alert alert-info py-1 px-2'>Enter phone (stub)</div>";
    } else {
      c.innerHTML = '';
      c.style.display = 'none';
    }
  }

  async function login(e) {
    e.preventDefault();
    clear($('login-err-alert'));
    const form = $('login-form');
    const email = form.email.value;
    const password = form.password.value;
    if (!email || !password) {
      err($('login-err-alert'), 'Email & password required (422)');
      focusFirstInvalid([form.email, form.password]);
      return;
    }
    form.querySelector('button').disabled = true;
    try {
      const r = await fetch('/login', {
        method: 'POST',
        headers: {'Content-Type': 'application/json'},
        body: JSON.stringify({email, password})
      });
      let d = {};
      try { d = await r.json(); } catch {}
      if (r.status === 206) {
        const attemptId = d.loginAttemptId || d.login_attempt_id || '';
        $('2fa-form').email.value = email;
        $('2fa-form').login_attempt_id.value = attemptId;
        form.reset();
        swap(VIEWS.TWO_FA);
      } else if (r.ok) {
        form.reset();
        showToast('Success', 'Logged in');
        loggedIn(email);
        swap(VIEWS.LOGIN);
      } else {
        let friendly = 'Login failed.';
        if (r.status === 400) friendly = 'Invalid email or password format.';
        else if (r.status === 401) friendly = 'Incorrect email or password.';
        else if (r.status === 422) friendly = 'Missing or malformed fields.';
        err($('login-err-alert'), `${friendly} (HTTP ${r.status})`);
        focusFirstInvalid([form.email, form.password]);
      }
    } catch (ex) {
      err($('login-err-alert'), 'Network error');
    } finally {
      form.querySelector('button').disabled = false;
    }
  }

  async function signup(e) {
    e.preventDefault();
    clear($('signup-err-alert'));
    const form = $('signup-form');
    const email = form.email.value;
    const password = form.password.value;
    const requires2FA = form.twoFA.checked;
    if (!email || !password) {
      err($('signup-err-alert'), 'Email & password required (422)');
      focusFirstInvalid([form.email, form.password]);
      return;
    }
    form.querySelector('button').disabled = true;
    try {
      await api('/signup', {method: 'POST', body: {email, password, requires2FA}});
      form.reset();
      showToast('Success', 'Account created, please log in.');
      swap(VIEWS.LOGIN);
    } catch (er) {
      err($('signup-err-alert'), er.message + (er.status ? ` (HTTP ${er.status})` : ''));
    } finally {
      form.querySelector('button').disabled = false;
    }
  }

  async function verify2FA(e) {
    e.preventDefault();
    clear($('2fa-err-alert'));
    const form = $('2fa-form');
    const email = form.email.value;
    const id = form.login_attempt_id.value;
    const code = form.email_code.value;
    if (!code) {
      err($('2fa-err-alert'), 'Code required (422)');
      focusFirstInvalid([form.email_code]);
      return;
    }
    form.querySelector('button').disabled = true;
    try {
      await api('/verify-2fa', {method: 'POST', body: {email, loginAttemptId: id, '2FACode': code}});
      form.reset();
      showToast('Success', 'Logged in');
      loggedIn(email);
      swap(VIEWS.LOGIN);
    } catch (er) {
      err($('2fa-err-alert'), er.message + (er.status ? ` (HTTP ${er.status})` : ''));
    } finally {
      form.querySelector('button').disabled = false;
    }
  }

  async function forgotSubmit(e) {
    e.preventDefault();
    const step1 = $('forgot-password-step1').style.display !== 'none';
    if (step1) {
      clear($('forgot-password-err-alert'));
      const form = $('forgot-password-form');
      const email = form.email.value;
      if (!email) {
        err($('forgot-password-err-alert'), 'Email required (422)');
        focusFirstInvalid([form.email]);
        return;
      }
      form.querySelector('button').disabled = true;
      try {
        const d = await api('/forgot-password', {method: 'POST', body: {email}});
        hide($('forgot-password-step1'));
        show($('forgot-password-step2'));
        $('forgot-password-form-step2').email.value = email;
        $('forgot-password-form-step2').login_attempt_id.value = d.loginAttemptId || d.login_attempt_id || '';
      } catch (er) {
        err($('forgot-password-err-alert'), er.message);
      } finally {
        form.querySelector('button').disabled = false;
      }
    } else {
      clear($('forgot-password-err-alert-step2'));
      const form = $('forgot-password-form-step2');
      const email = form.email.value;
      const id = form.login_attempt_id.value;
      const code = form.reset_code.value;
      const np = form.new_password.value;
      if (!code || !np) {
        err($('forgot-password-err-alert-step2'), 'Code & new password required (422)');
        focusFirstInvalid([!code ? form.reset_code : form.new_password]);
        return;
      }
      form.querySelector('button').disabled = true;
      try {
        await api('/reset-password', {method: 'POST', body: {email, loginAttemptId: id, code, new_password: np}});
        showToast('Success', 'Password reset. You can now log in.');
        swap(VIEWS.LOGIN);
        resetForgot();
      } catch (er) {
        err($('forgot-password-err-alert-step2'), er.message);
      } finally {
        form.querySelector('button').disabled = false;
      }
    }
  }

  function resetForgot() {
    clear($('forgot-password-err-alert'));
    clear($('forgot-password-err-alert-step2'));
    show($('forgot-password-step1'));
    hide($('forgot-password-step2'));
    $('forgot-password-form').reset();
    $('forgot-password-form-step2').reset();
  }

  async function toggle2FA() {
    const enable = $('2fa-toggle').checked;
    if (!confirm(`Are you sure you want to ${enable ? 'enable' : 'disable'} 2FA?`)) {
      $('2fa-toggle').checked = !enable;
      return;
    }
    clear($('settings-err-alert'));
    try {
      await api('/account/settings', {method: 'PATCH', headers: {...(st.user ? {'x-user-email': st.user} : {})}, body: {requires2FA: enable, twoFAMethod: $('2fa-method-select').value}});
      st.twoFA.enabled = enable;
      st.twoFA.method = $('2fa-method-select').value;
      update2FABadge();
      showToast('Success', `2FA ${enable ? 'enabled' : 'disabled'}`);
    } catch (er) {
      err($('settings-err-alert'), er.message);
      $('2fa-toggle').checked = !enable;
    }
  }

  async function delAccount() {
    if (!st.user) return alert('No user');
    if (!confirm('Delete account permanently?')) return;
    $('delete-account-btn').disabled = true;
    try {
      await api('/delete-account', {method: 'DELETE', headers: {'x-user-email': st.user}});
      showToast('Success', 'Account deleted');
      loggedOut();
      swap(VIEWS.LOGIN);
    } catch (er) {
      alert(er.message || 'Delete failed');
    } finally {
      $('delete-account-btn').disabled = false;
    }
  }

  function change2FAMethod() {
    st.twoFA.method = $('2fa-method-select').value;
    update2FABadge();
  }

  function nav() {
    $('signup-link').addEventListener('click', e => { e.preventDefault(); swap(VIEWS.SIGNUP); });
    $('signup-login-link').addEventListener('click', e => { e.preventDefault(); swap(VIEWS.LOGIN); });
    $('2fa-login-link').addEventListener('click', e => { e.preventDefault(); swap(VIEWS.LOGIN); clear($('2fa-err-alert')); });
    $('forgot-password-link').addEventListener('click', e => { e.preventDefault(); swap(VIEWS.FORGOT); resetForgot(); });
    $('forgot-password-login-link').addEventListener('click', e => { e.preventDefault(); swap(VIEWS.LOGIN); resetForgot(); });
    $('settings-login-link').addEventListener('click', e => { e.preventDefault(); loggedOut(); swap(VIEWS.LOGIN); });
  }

  function start() {
    swap(VIEWS.LOGIN);
    loggedOut();
    nav();
    $('login-form').addEventListener('submit', login);
    $('signup-form').addEventListener('submit', signup);
    $('2fa-form').addEventListener('submit', verify2FA);
    $('forgot-password-form').addEventListener('submit', forgotSubmit);
    $('forgot-password-form-step2').addEventListener('submit', forgotSubmit);
    $('2fa-toggle').addEventListener('change', toggle2FA);
    $('2fa-method-select').addEventListener('change', change2FAMethod);
    $('delete-account-btn').addEventListener('click', delAccount);
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', start);
  } else {
    start();
  }
})();