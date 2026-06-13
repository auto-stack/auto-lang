# Auto-generated from concat.at — do not edit

extends Node

var robot_name: String = "Robi"

func _ready():
	print("Hi, " + robot_name + "!")
	var msg: String = "The robot is " + robot_name
	print(msg)
