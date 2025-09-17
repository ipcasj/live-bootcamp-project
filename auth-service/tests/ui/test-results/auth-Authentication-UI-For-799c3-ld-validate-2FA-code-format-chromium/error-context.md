# Page snapshot

```yaml
- main [ref=e3]:
  - main [ref=e4]:
    - generic [ref=e6]:
      - heading "Welcome back" [level=1] [ref=e7]
      - paragraph [ref=e8]: Sign in to your account to continue
      - alert [ref=e9]:
        - generic [ref=e10]: ⚠️
        - generic [ref=e11]: Failed to fetch
      - generic [ref=e12]:
        - generic [ref=e13]:
          - generic [ref=e14]: Email address
          - generic [ref=e15]:
            - generic [ref=e16]: 📧
            - textbox "Email address" [active] [ref=e17]: test@example.com
        - generic [ref=e18]:
          - generic [ref=e19]: Password
          - generic [ref=e20]:
            - generic [ref=e21]: 🔒
            - textbox "Password" [ref=e22]: password123
        - button "Sign in" [ref=e23] [cursor=pointer]:
          - generic [ref=e24] [cursor=pointer]: Sign in
      - generic [ref=e25]:
        - link "Forgot your password?" [ref=e27] [cursor=pointer]:
          - /url: "#"
        - generic [ref=e28]:
          - text: Don't have an account?
          - link "Create one" [ref=e29] [cursor=pointer]:
            - /url: "#"
```