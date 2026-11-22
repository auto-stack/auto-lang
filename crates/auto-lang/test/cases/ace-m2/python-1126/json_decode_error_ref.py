# Independent native-Python reference for the `json-decode-error-uncaught`
# case (PLAN-1126). No ACE is involved and the exception is deliberately NOT
# caught: the interpreter itself must fail, naming the original class.
import json

data = json.loads("{")
print(len(data["values"]))
