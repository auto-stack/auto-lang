# Auto-generated from string_array.at — do not edit

extends Node

var combo: Array[String] = ["jab", "jab", "uppercut"]

func play_animation(name: int):
	print(name)

func _ready():
	for animation_name in combo:
		play_animation(animation_name)
