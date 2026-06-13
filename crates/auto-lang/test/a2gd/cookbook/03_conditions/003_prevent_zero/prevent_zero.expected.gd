# Auto-generated from prevent_zero.at — do not edit

extends Node

var health: int = 20

func take_damage(amount: int):
	health = health - amount
	if health < 0:
		health = 0

func _ready():
	take_damage(10)
	print(health)
	take_damage(50)
	print(health)
