# Page snapshot

```yaml
- main [ref=e3]:
  - main [ref=e4]:
    - generic [ref=e6]:
      - heading "Welcome back" [level=1] [ref=e7]
      - paragraph [ref=e8]: Sign in to your account to continue
      - generic [ref=e9]:
        - generic [ref=e10]:
          - generic [ref=e11]: Email address
          - generic [ref=e12]:
            - generic [ref=e13]: 📧
            - textbox "Email address" [ref=e14]
            - generic [ref=e15]: Email is required
        - generic [ref=e16]:
          - generic [ref=e17]: Password
          - generic [ref=e18]:
            - generic [ref=e19]: 🔒
            - textbox "Password" [ref=e20]
        - button "Sign in" [ref=e21] [cursor=pointer]:
          - generic [ref=e22] [cursor=pointer]: Sign in
      - generic [ref=e23]:
        - link "Forgot your password?" [ref=e25] [cursor=pointer]:
          - /url: "#"
        - generic [ref=e26]:
          - text: Don't have an account?
          - link "Create one" [active] [ref=e27] [cursor=pointer]:
            - /url: "#"
```