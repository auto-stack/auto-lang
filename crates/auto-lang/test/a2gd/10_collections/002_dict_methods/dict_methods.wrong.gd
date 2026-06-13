# Auto-generated from dict_methods.at — do not edit

extends Node

func _ready():
	var scores = {"alice": 90, "bob": 85}
	scores["charlie"] = 95
	var score = scores.get("alice")
	var found = "bob" in scores
	var all_keys = scores.keys()
	print(score)
	print(found)
	print(all_keys)
