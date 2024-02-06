# Floppa Auth
silly little, tries to adhere to oauth2 specification with my own little spin on it

## 'GET '
### Parameters
### Description
### Response





## `GET oauth2/authorize`
### Paremeters
  - `response_type`: usually **code** but specifies what the response should be
  - `client_id`: your apps client id
  - `redirect_uri`: where floppa-auth should redirect back to
  - `scope`: what data your app requires to access **CURRENTLY UNUSED**

### Description
redirecting the user to this url will start the auth procedure, where it will either ask the user to sign in if they have not used the service before or have invalid cookie it will go through the signup process 
### Respone
redirects to `redirect_uri/callback?code=AUTHORIZATION_CODE` if successful




## `POST /oauth2/token` 
### Parameters
  - `client_id`: your apps client id
  - `client_secret`: your apps client secret
  - `grant_type`: usually **authorization_code** idk what else
  - `code`: the authorization code previously requested

### Description
after getting authorization code, in your server BACKEND you verify it and get an access token to use in other things
### Response
if everything was valid, will return the below json
  ```json
  {
    "access_token": "ACCESS_TOKEN",
    "token_type": "bearer",
    "expires_in": 69,
    "refresh_token": "REFRESH_TOKEN",
    "scope": "read",
    "uid": 69,
    "info": {
        "username":"floppa",
        "email":"mark@thefunkybunch.com"8
    }
 } 
  ```
  if everything is invalid it will return statuscode 418



















credits to digitalocean docs for the endpoints
