# Page snapshot

```yaml
- main [ref=e3]:
  - main [ref=e4]:
    - generic [ref=e6]:
      - heading "Create account" [level=1] [ref=e7]
      - paragraph [ref=e8]: Get started with your new account
      - alert [ref=e9]:
        - generic [ref=e10]: ⚠️
        - generic [ref=e11]: Failed to fetch
      - generic [ref=e12]:
        - generic [ref=e13]:
          - generic [ref=e14]: Email address
          - generic [ref=e15]:
            - generic [ref=e16]: 📧
            - textbox "Email address" [ref=e17]: test1758070218048@example.com
        - generic [ref=e18]:
          - generic [ref=e19]: Password
          - generic [ref=e20]:
            - generic [ref=e21]: 🔒
            - textbox "Password" [ref=e22]: password123
          - generic [ref=e23]: Password must be at least 8 characters long
        - generic [ref=e24]:
          - checkbox "Enable two-factor authentication (recommended)" [checked] [ref=e25] [cursor=pointer]: ✓
          - generic [ref=e26] [cursor=pointer]: Enable two-factor authentication (recommended)
        - button "Create account" [ref=e27] [cursor=pointer]:
          - generic [ref=e28] [cursor=pointer]: Create account
      - generic [ref=e30]:
        - text: Already have an account?
        - link "Sign in" [ref=e31] [cursor=pointer]:
          - /url: "#"
```