# floppa auth
authentication for all **gosher studios** trademark software starting now



how to use if you are trying to be a user.
create an app by asking the hoster of the floppa auth to make you a thing through a POST Request to URL/meow?name=name&url=url
url must be formatted like https://url.com? or it won't work
there will be output to console of the secret

then when need login user - redirect user to url/?appid={your apps name}&secret={your secret(this is slightly unsafe i will fix)} and then 
user will be redirected to {callback url}?id={id}
this id is their current session id which will last for 14 days
add it as a cookie or into local storage idrc


to check if session is still valid send a GET Request to URL/unknown
i will finish this in a sec lemme lunch :3
