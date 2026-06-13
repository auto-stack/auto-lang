# Auto-generated from append_pop.at — do not edit

extends Node

var waiting: Variant = ["sword", "shield"]

var completed = []

func complete_order():
	var item = waiting.pop()
	completed.append(item)

func _ready():
	print(len(waiting))
	complete_order()
	print(len(waiting))
	print(len(completed))
	print(completed)
