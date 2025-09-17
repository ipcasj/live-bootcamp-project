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
            - textbox "Email address" [active] [ref=e14]
        - generic [ref=e15]:
          - generic [ref=e16]: Password
          - generic [ref=e17]:
            - generic [ref=e18]: 🔒
            - textbox "Password" [ref=e19]
        - button "Sign in" [ref=e20] [cursor=pointer]:
          - generic [ref=e21] [cursor=pointer]: Sign in
      - generic [ref=e22]:
        - link "Forgot your password?" [ref=e24] [cursor=pointer]:
          - /url: "#"
        - generic [ref=e25]:
          - text: Don't have an account?
          - link "Create one" [ref=e26] [cursor=pointer]:
            - /url: "#"
```