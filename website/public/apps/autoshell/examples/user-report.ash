var users = > cat users.json | from_json
var adults = 0

for user in users {
    if user.age >= 30 {
        print(user.name + " (" + user.age.str() + ")")
        adults = adults + 1
    }
}

print("matching users: " + adults.str())
