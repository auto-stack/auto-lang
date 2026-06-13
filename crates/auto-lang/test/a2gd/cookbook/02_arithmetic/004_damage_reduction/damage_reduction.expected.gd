# Auto-generated from damage_reduction.at — do not edit

extends Node

var level: int = 3

var health: int = 100

func take_damage(amount: int):
	if level > 2:
		amount = amount * 0.5
	health = health - amount
	if health < 0:
		health = 0

func _ready():
	take_damage(60)
	print(health)
	take_damage(40)
	print(health)
