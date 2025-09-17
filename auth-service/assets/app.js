/**
 * Modern Authentication UI Application
 * Enhanced UX with progressive validation, loading states, and accessibility
 */

class AuthApp {
  constructor() {
    this.state = {
      currentView: 'login',
      user: null,
      isAuthenticated: false,
      twoFA: { enabled: false, method: 'Email' },
      isLoading: false
    };
    
    this.views = {
      LOGIN: 'login-section',
      SIGNUP: 'signup-section',
      TWO_FA: '2fa-section',
      FORGOT: 'forgot-password-section',
      SETTINGS: 'account-settings-section'
    };
    
    this.elements = {};
    this.validationRules = {
      email: /^[^\s@]+@[^\s@]+\.[^\s@]+$/,
      password: /.{8,}/
    };
    
    this.init();
  }

  // Initialize the application
  init() {
    this.cacheElements();
    this.bindEvents();
    this.setupProgressiveValidation();
    this.setupAccessibility();
    this.showView(this.views.LOGIN);
    this.updateUIState();
    this.startAuthCheck(); // Start periodic authentication checking
  }

  // Start periodic authentication status checking
  startAuthCheck() {
    // Check auth status every 5 minutes
    setInterval(() => {
      if (this.state.isAuthenticated) {
        this.checkAuthStatus();
      }
    }, 5 * 60 * 1000);
  }

  // Cache DOM elements for performance
  cacheElements() {
    // Views
    Object.values(this.views).forEach(viewId => {
      this.elements[viewId] = document.getElementById(viewId);
    });
    
    // Forms
    this.elements.loginForm = document.getElementById('login-form');
    this.elements.signupForm = document.getElementById('signup-form');
    this.elements.twoFAForm = document.getElementById('2fa-form');
    this.elements.forgotForm = document.getElementById('forgot-password-form');
    this.elements.forgotForm2 = document.getElementById('forgot-password-form-step2');
    
    // Alert containers
    this.elements.alerts = {
      login: document.getElementById('login-err-alert'),
      signup: document.getElementById('signup-err-alert'),
      twoFA: document.getElementById('2fa-err-alert'),
      forgot: document.getElementById('forgot-password-err-alert'),
      forgot2: document.getElementById('forgot-password-err-alert-step2'),
      settings: document.getElementById('settings-err-alert')
    };
    
    // Navigation elements
    this.elements.header = document.getElementById('app-header');
    this.elements.userEmail = document.getElementById('user-email');
    this.elements.userAvatar = document.getElementById('user-avatar');
    this.elements.twoFABadge = document.getElementById('2fa-badge');
    this.elements.twoFAToggle = document.getElementById('2fa-toggle');
    this.elements.twoFAMethod = document.getElementById('2fa-method-select');
    
    // Toast container
    this.elements.toastContainer = document.getElementById('toast-container');
  }

  // Bind event listeners
  bindEvents() {
    // Form submissions
    this.elements.loginForm?.addEventListener('submit', (e) => this.handleLogin(e));
    this.elements.signupForm?.addEventListener('submit', (e) => this.handleSignup(e));
    this.elements.twoFAForm?.addEventListener('submit', (e) => this.handleTwoFA(e));
    this.elements.forgotForm?.addEventListener('submit', (e) => this.handleForgotPassword(e));
    this.elements.forgotForm2?.addEventListener('submit', (e) => this.handlePasswordReset(e));
    
    // Navigation links
    this.bindNavigationEvents();
    
    // Settings
    this.elements.twoFAToggle?.addEventListener('change', () => this.toggleTwoFA());
    this.elements.twoFAMethod?.addEventListener('change', () => this.changeTwoFAMethod());
    document.getElementById('delete-account-btn')?.addEventListener('click', () => this.deleteAccount());
  }

  // Bind navigation event listeners
  bindNavigationEvents() {
    const navBindings = [
      { id: 'signup-link', view: this.views.SIGNUP },
      { id: 'signup-login-link', view: this.views.LOGIN },
      { id: 'forgot-password-link', view: this.views.FORGOT, callback: () => this.resetForgotPasswordForm() },
      { id: 'forgot-password-login-link', view: this.views.LOGIN, callback: () => this.resetForgotPasswordForm() },
      { id: '2fa-login-link', view: this.views.LOGIN, callback: () => this.clearAlert('twoFA') },
      { id: 'settings-login-link', callback: () => this.logout() }
    ];
    
    navBindings.forEach(({ id, view, callback }) => {
      const element = document.getElementById(id);
      if (element) {
        element.addEventListener('click', (e) => {
          e.preventDefault();
          if (callback) callback();
          if (view) this.showView(view);
        });
      }
    });
  }

  // Setup progressive form validation
  setupProgressiveValidation() {
    const forms = [
      this.elements.loginForm,
      this.elements.signupForm,
      this.elements.twoFAForm,
      this.elements.forgotForm,
      this.elements.forgotForm2
    ];
    
    forms.forEach(form => {
      if (!form) return;
      
      const inputs = form.querySelectorAll('input[required]');
      inputs.forEach(input => {
        input.addEventListener('blur', () => this.validateField(input));
        input.addEventListener('input', () => this.clearFieldError(input));
      });
    });
  }

  // Setup accessibility features
  setupAccessibility() {
    // Add ARIA labels and descriptions
    document.querySelectorAll('.form-input').forEach(input => {
      const label = document.querySelector(`label[for="${input.id}"]`);
      if (label) {
        input.setAttribute('aria-labelledby', label.id || `${input.id}-label`);
      }
    });
    
    // Announce view changes to screen readers
    document.querySelectorAll('.auth-section').forEach(section => {
      section.setAttribute('role', 'main');
      section.setAttribute('aria-live', 'polite');
    });
  }

  // Validate individual form field
  validateField(input) {
    const value = input.value.trim();
    const fieldName = input.name || input.id;
    let isValid = true;
    let errorMessage = '';

    // Required field check
    if (input.hasAttribute('required') && !value) {
      isValid = false;
      errorMessage = `${this.capitalizeFirstLetter(fieldName)} is required`;
    }
    // Email validation
    else if (input.type === 'email' && value && !this.validationRules.email.test(value)) {
      isValid = false;
      errorMessage = 'Please enter a valid email address';
    }
    // Password validation
    else if (input.type === 'password' && value && !this.validationRules.password.test(value)) {
      isValid = false;
      errorMessage = 'Password must be at least 8 characters long';
    }
    // Pattern validation
    else if (input.pattern && value && !new RegExp(input.pattern).test(value)) {
      isValid = false;
      errorMessage = 'Please enter a valid format';
    }

    this.setFieldValidationState(input, isValid, errorMessage);
    return isValid;
  }

  // Set field validation state
  setFieldValidationState(input, isValid, errorMessage = '') {
    const inputGroup = input.closest('.input-group') || input.parentElement;
    
    if (isValid) {
      input.classList.remove('error');
      input.removeAttribute('aria-describedby');
      this.removeFieldError(inputGroup);
    } else {
      input.classList.add('error');
      this.showFieldError(inputGroup, errorMessage);
      input.setAttribute('aria-describedby', `${input.id}-error`);
    }
  }

  // Show field-level error
  showFieldError(container, message) {
    this.removeFieldError(container);
    
    const errorElement = document.createElement('div');
    errorElement.className = 'field-error text-xs mt-2';
    errorElement.style.color = 'var(--error-color)';
    errorElement.textContent = message;
    errorElement.id = `${container.querySelector('input').id}-error`;
    
    container.appendChild(errorElement);
  }

  // Remove field-level error
  removeFieldError(container) {
    const existing = container.querySelector('.field-error');
    if (existing) existing.remove();
  }

  // Clear field error on input
  clearFieldError(input) {
    input.classList.remove('error');
    input.removeAttribute('aria-describedby');
    const container = input.closest('.input-group') || input.parentElement;
    this.removeFieldError(container);
  }

  // Validate entire form
  validateForm(form) {
    const inputs = form.querySelectorAll('input[required]');
    let isValid = true;
    let firstInvalidInput = null;

    inputs.forEach(input => {
      const fieldValid = this.validateField(input);
      if (!fieldValid && !firstInvalidInput) {
        firstInvalidInput = input;
      }
      isValid = isValid && fieldValid;
    });

    if (firstInvalidInput) {
      firstInvalidInput.focus();
    }

    return isValid;
  }

  // Show loading state on button
  setButtonLoading(button, isLoading) {
    if (isLoading) {
      button.classList.add('btn-loading');
      button.disabled = true;
      button.setAttribute('aria-busy', 'true');
    } else {
      button.classList.remove('btn-loading');
      button.disabled = false;
      button.removeAttribute('aria-busy');
    }
  }

  // API request wrapper with loading states and auth handling
  async apiRequest(path, options = {}) {
    const config = {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
        ...(this.state.user && { 'x-user-email': this.state.user })
      },
      credentials: 'include', // Include cookies for JWT auth
      ...options
    };

    if (config.body && typeof config.body === 'object') {
      config.body = JSON.stringify(config.body);
    }

    try {
      const response = await fetch(path, config);
      let data = {};
      
      try {
        data = await response.json();
      } catch (e) {
        // Response might not have JSON body
      }

      // Handle authentication errors
      if (response.status === 401) {
        if (this.state.isAuthenticated) {
          this.showToast('Session Expired', 'Please sign in again', 'warning');
          this.logout();
        }
        throw new Error('Authentication required');
      }

      if (!response.ok) {
        const errorMessage = data.error || data.message || `HTTP ${response.status}`;
        throw new Error(errorMessage);
      }

      return { data, status: response.status };
    } catch (error) {
      throw new Error(error.message || 'Network error occurred');
    }
  }

  // Handle login form submission
  async handleLogin(event) {
    event.preventDefault();
    
    const form = event.target;
    const submitButton = form.querySelector('button[type="submit"]');
    
    if (!this.validateForm(form)) return;
    
    this.clearAlert('login');
    this.setButtonLoading(submitButton, true);
    
    try {
      const formData = new FormData(form);
      const email = formData.get('email');
      const password = formData.get('password');
      
      const response = await fetch('/login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email, password })
      });
      
      let data = {};
      try {
        data = await response.json();
      } catch (e) {}
      
      if (response.status === 206) {
        // 2FA required
        const attemptId = data.loginAttemptId || data.login_attempt_id || '';
        this.elements.twoFAForm.email.value = email;
        this.elements.twoFAForm.login_attempt_id.value = attemptId;
        form.reset();
        this.showView(this.views.TWO_FA);
        this.showToast('2FA Required', 'Check your email for the verification code', 'warning');
      } else if (response.ok) {
        // Successful login
        form.reset();
        this.setAuthenticatedUser(email);
        this.showToast('Welcome!', 'You have successfully signed in');
        this.showView(this.views.SETTINGS);
      } else {
        const errorMessage = data.error || data.message || 'Invalid credentials';
        this.showAlert('login', errorMessage);
      }
    } catch (error) {
      this.showAlert('login', error.message);
    } finally {
      this.setButtonLoading(submitButton, false);
    }
  }

  // Handle signup form submission
  async handleSignup(event) {
    event.preventDefault();
    
    const form = event.target;
    const submitButton = form.querySelector('button[type="submit"]');
    
    if (!this.validateForm(form)) return;
    
    this.clearAlert('signup');
    this.setButtonLoading(submitButton, true);
    
    try {
      const formData = new FormData(form);
      const email = formData.get('email');
      const password = formData.get('password');
      const requires2FA = formData.get('twoFA') === 'on';
      
      const { data } = await this.apiRequest('/signup', {
        method: 'POST',
        body: { email, password, requires2FA }
      });
      
      form.reset();
      this.showToast('Account Created!', 'Please sign in with your new account');
      this.showView(this.views.LOGIN);
    } catch (error) {
      this.showAlert('signup', error.message);
    } finally {
      this.setButtonLoading(submitButton, false);
    }
  }

  // Handle 2FA verification
  async handleTwoFA(event) {
    event.preventDefault();
    
    const form = event.target;
    const submitButton = form.querySelector('button[type="submit"]');
    
    if (!this.validateForm(form)) return;
    
    this.clearAlert('twoFA');
    this.setButtonLoading(submitButton, true);
    
    try {
      const formData = new FormData(form);
      const email = formData.get('email');
      const loginAttemptId = formData.get('login_attempt_id');
      const code = formData.get('email_code');
      
      await this.apiRequest('/verify-2fa', {
        method: 'POST',
        body: { email, loginAttemptId, '2FACode': code }
      });
      
      form.reset();
      this.setAuthenticatedUser(email);
      this.showToast('Success!', 'You have been logged in');
      this.showView(this.views.SETTINGS);
    } catch (error) {
      this.showAlert('twoFA', error.message);
    } finally {
      this.setButtonLoading(submitButton, false);
    }
  }

  // Handle forgot password (step 1)
  async handleForgotPassword(event) {
    event.preventDefault();
    
    const form = event.target;
    const submitButton = form.querySelector('button[type="submit"]');
    
    if (!this.validateForm(form)) return;
    
    this.clearAlert('forgot');
    this.setButtonLoading(submitButton, true);
    
    try {
      const formData = new FormData(form);
      const email = formData.get('email');
      
      const { data } = await this.apiRequest('/forgot-password', {
        method: 'POST',
        body: { email }
      });
      
      // Move to step 2
      document.getElementById('forgot-password-step1').classList.add('hidden');
      document.getElementById('forgot-password-step2').classList.remove('hidden');
      
      this.elements.forgotForm2.email.value = email;
      this.elements.forgotForm2.login_attempt_id.value = data.loginAttemptId || data.login_attempt_id || '';
      
      this.showToast('Reset Code Sent', 'Check your email for the reset code');
    } catch (error) {
      this.showAlert('forgot', error.message);
    } finally {
      this.setButtonLoading(submitButton, false);
    }
  }

  // Handle password reset (step 2)
  async handlePasswordReset(event) {
    event.preventDefault();
    
    const form = event.target;
    const submitButton = form.querySelector('button[type="submit"]');
    
    if (!this.validateForm(form)) return;
    
    this.clearAlert('forgot2');
    this.setButtonLoading(submitButton, true);
    
    try {
      const formData = new FormData(form);
      const email = formData.get('email');
      const loginAttemptId = formData.get('login_attempt_id');
      const code = formData.get('reset_code');
      const newPassword = formData.get('new_password');
      
      await this.apiRequest('/reset-password', {
        method: 'POST',
        body: { email, loginAttemptId, code, new_password: newPassword }
      });
      
      this.showToast('Password Reset!', 'You can now sign in with your new password');
      this.showView(this.views.LOGIN);
      this.resetForgotPasswordForm();
    } catch (error) {
      this.showAlert('forgot2', error.message);
    } finally {
      this.setButtonLoading(submitButton, false);
    }
  }

  // Toggle 2FA setting
  async toggleTwoFA() {
    const enable = this.elements.twoFAToggle.checked;
    
    if (!confirm(`Are you sure you want to ${enable ? 'enable' : 'disable'} 2FA?`)) {
      this.elements.twoFAToggle.checked = !enable;
      return;
    }
    
    this.clearAlert('settings');
    
    try {
      await this.apiRequest('/account/settings', {
        method: 'PATCH',
        body: {
          requires2FA: enable,
          twoFAMethod: this.elements.twoFAMethod.value
        }
      });
      
      this.state.twoFA.enabled = enable;
      this.state.twoFA.method = this.elements.twoFAMethod.value;
      this.updateTwoFABadge();
      this.showToast('Settings Updated', `2FA ${enable ? 'enabled' : 'disabled'}`);
    } catch (error) {
      this.showAlert('settings', error.message);
      this.elements.twoFAToggle.checked = !enable;
    }
  }

  // Change 2FA method
  changeTwoFAMethod() {
    this.state.twoFA.method = this.elements.twoFAMethod.value;
  }

  // Delete account
  async deleteAccount() {
    const confirmation = prompt('Type "DELETE" to confirm account deletion:');
    if (confirmation !== 'DELETE') return;
    
    this.clearAlert('settings');
    
    try {
      await this.apiRequest('/delete-account', { method: 'DELETE' });
      this.showToast('Account Deleted', 'Your account has been permanently deleted');
      this.logout();
    } catch (error) {
      this.showAlert('settings', error.message);
    }
  }

  // Set authenticated user state
  setAuthenticatedUser(email) {
    this.state.user = email;
    this.state.isAuthenticated = true;
    this.loadAccountSettings(); // Load current settings when user logs in
    this.updateUIState();
  }

  // Load account settings from backend
  async loadAccountSettings() {
    try {
      const { data } = await this.apiRequest('/account/settings', { method: 'GET' });
      this.state.twoFA.enabled = data.requires2FA || false;
      this.state.twoFA.method = data.twoFAMethod || 'Email';
      this.updateTwoFASettings();
    } catch (error) {
      console.warn('Failed to load account settings:', error.message);
      // Use default values if loading fails
    }
  }

  // Check if user is still authenticated (token validation)
  async checkAuthStatus() {
    if (!this.state.isAuthenticated) return false;
    
    try {
      await this.apiRequest('/account/settings', { method: 'GET' });
      return true; // If we can access protected route, we're still authenticated
    } catch (error) {
      // If token is invalid, logout user
      this.logout();
      return false;
    }
  }

  // Logout user
  async logout() {
    try {
      // Call backend logout to invalidate server-side session
      await this.apiRequest('/logout', { method: 'POST' });
    } catch (error) {
      console.warn('Logout API call failed:', error.message);
      // Continue with client-side logout even if server call fails
    }
    
    // Clear client-side state
    this.state.user = null;
    this.state.isAuthenticated = false;
    this.state.twoFA = { enabled: false, method: 'Email' };
    this.updateUIState();
    this.showView(this.views.LOGIN);
    this.showToast('Signed Out', 'You have been successfully signed out');
  }

  // Update UI state based on authentication
  updateUIState() {
    if (this.state.isAuthenticated) {
      this.elements.header.style.display = 'block';
      this.elements.userEmail.textContent = this.state.user;
      this.elements.userAvatar.textContent = this.state.user.charAt(0).toUpperCase();
      this.updateTwoFABadge();
      this.updateTwoFASettings();
    } else {
      this.elements.header.style.display = 'none';
    }
  }

  // Update 2FA badge visibility
  updateTwoFABadge() {
    if (this.state.twoFA.enabled) {
      this.elements.twoFABadge.classList.remove('hidden');
    } else {
      this.elements.twoFABadge.classList.add('hidden');
    }
  }

  // Update 2FA settings in the UI
  updateTwoFASettings() {
    if (this.elements.twoFAToggle) {
      this.elements.twoFAToggle.checked = this.state.twoFA.enabled;
    }
    if (this.elements.twoFAMethod) {
      this.elements.twoFAMethod.value = this.state.twoFA.method;
    }
  }

  // Show specific view
  showView(viewId) {
    // Hide all views
    Object.values(this.views).forEach(id => {
      const element = this.elements[id];
      if (element) {
        element.classList.add('hidden');
      }
    });
    
    // Show target view
    const targetView = this.elements[viewId];
    if (targetView) {
      targetView.classList.remove('hidden');
      this.state.currentView = viewId;
      
      // Focus first input in the view
      setTimeout(() => {
        const firstInput = targetView.querySelector('input');
        if (firstInput) firstInput.focus();
      }, 100);
    }
  }

  // Show alert message
  showAlert(type, message) {
    const alertElement = this.elements.alerts[type];
    if (!alertElement) return;
    
    const contentElement = alertElement.querySelector('.alert-content') || alertElement;
    contentElement.textContent = message;
    alertElement.classList.remove('hidden');
    
    // Announce to screen readers
    alertElement.setAttribute('aria-live', 'assertive');
    
    // Auto-hide after 10 seconds
    setTimeout(() => {
      this.clearAlert(type);
    }, 10000);
  }

  // Clear alert message
  clearAlert(type) {
    const alertElement = this.elements.alerts[type];
    if (alertElement) {
      alertElement.classList.add('hidden');
      alertElement.removeAttribute('aria-live');
    }
  }

  // Show toast notification
  showToast(title, message, variant = 'success', duration = 4000) {
    const toast = document.createElement('div');
    toast.className = `toast toast-${variant}`;
    
    const iconMap = {
      success: '✅',
      error: '❌',
      warning: '⚠️',
      info: 'ℹ️'
    };
    
    toast.innerHTML = `
      <div class="toast-icon">${iconMap[variant] || iconMap.info}</div>
      <div class="toast-content">
        <div class="toast-title">${this.escapeHtml(title)}</div>
        <div class="toast-message">${this.escapeHtml(message)}</div>
      </div>
      <button class="toast-close" aria-label="Close notification">×</button>
    `;
    
    // Add close functionality
    toast.querySelector('.toast-close').addEventListener('click', () => {
      this.removeToast(toast);
    });
    
    this.elements.toastContainer.appendChild(toast);
    
    // Auto-remove after duration
    setTimeout(() => {
      this.removeToast(toast);
    }, duration);
  }

  // Remove toast notification
  removeToast(toast) {
    if (toast && toast.parentNode) {
      toast.style.animation = 'slideOut 0.3s ease-in forwards';
      setTimeout(() => {
        if (toast.parentNode) {
          toast.parentNode.removeChild(toast);
        }
      }, 300);
    }
  }

  // Reset forgot password form
  resetForgotPasswordForm() {
    this.clearAlert('forgot');
    this.clearAlert('forgot2');
    document.getElementById('forgot-password-step1').classList.remove('hidden');
    document.getElementById('forgot-password-step2').classList.add('hidden');
    this.elements.forgotForm?.reset();
    this.elements.forgotForm2?.reset();
  }

  // Utility: Escape HTML
  escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
  }

  // Utility: Capitalize first letter
  capitalizeFirstLetter(string) {
    return string.charAt(0).toUpperCase() + string.slice(1);
  }
}

// Add CSS animation for toast slide out
const style = document.createElement('style');
style.textContent = `
  @keyframes slideOut {
    to {
      transform: translateX(100%);
      opacity: 0;
    }
  }
`;
document.head.appendChild(style);

// Initialize app when DOM is ready
function initApp() {
  if (!window.authApp) {
    window.authApp = new AuthApp();
  }
}

if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', initApp);
} else {
  initApp();
}

// Also try to initialize after a short delay as fallback
setTimeout(() => {
  if (!window.authApp) {
    initApp();
  }
}, 100);