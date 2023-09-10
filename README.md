# floppa auth
authentication for all **gosher studios** trademark software starting now



# Usage Instructions

## Creating a User Account

If you're looking to become a user, follow these steps:

1. Create an app by sending a POST Request to the `URL/meow` endpoint with the following parameters:
   - `name`: Your desired app name.
   - `url`: The callback URL formatted as `https://url.com`.

   **Note:** Ensure that the `url` parameter is correctly formatted as `https://url.com` for it to work properly.



2. After successful creation, the hoster will receive a secret which will be outputted to the console.

## Logging In a User

To log in a user, follow these steps:

1. Redirect the user to `url/?appid={your app's name}&secret={your secret}`. 
   - **Note:** Using the secret in the URL is considered slightly unsafe; it will be addressed in future updates.

2. The user will be redirected to `{callback url}?id={id}`. This `id` represents their current session ID, which will remain valid for 14 days. You can store it as a cookie or in local storage.

## Checking Session Validity

To verify if a session is still valid, send a GET Request to the `URL/auth` endpoint with the following parameters:

- `ssid`: Session ID
- `secret`: App secret
- `name`: App name

If the request is successful, it will return a status code `200` with the body containing the username of the user. If not, it will return either `401` or `404` depending on the error.
