# Auto-generated from match.at — do not edit

extends Node

func _ready():
	var x: int = 3
	match x:
		0:
			print("zero")
		1:
			print("one")
		_:
			print("other")
