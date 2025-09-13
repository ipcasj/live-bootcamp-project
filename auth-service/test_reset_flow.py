import requests

BASE_URL = "http://localhost:3000"

def test_forgot_password(email):
    resp = requests.post(f"{BASE_URL}/forgot-password", json={"email": email})
    print("Forgot password response:", resp.status_code, resp.json())

def test_reset_password(token, new_password):
    resp = requests.post(f"{BASE_URL}/reset-password", json={"token": token, "new_password": new_password})
    print("Reset password response:", resp.status_code, resp.json())

if __name__ == "__main__":
    # 1. Replace with a real user email in your DB
    email = "test@example.com"
    test_forgot_password(email)
    # 2. Copy the token printed in your backend logs and paste below
    #token = "YXaT7nWGQlXb8EUyEfYuoYBXLYgFkjT8KMj3pzGwQn5yKrSB"
    #test_reset_password(token, "newpassword123")
