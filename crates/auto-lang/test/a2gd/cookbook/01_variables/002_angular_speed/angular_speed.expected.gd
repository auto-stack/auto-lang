# Auto-generated from angular_speed.at — do not edit

extends Node

var angular_speed: int = 4

func set_angular_speed(new_speed: int):
	angular_speed = new_speed

func _ready():
	print(angular_speed)
	set_angular_speed(8)
	print(angular_speed)
