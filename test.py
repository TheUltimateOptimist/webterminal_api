import requests

response = requests.post("http://localhost:3000/worktracker/sessions/add/1/4.5/3.4")
print(response.status_code)
print(response.text)