# Auto-generated from main.at.at — do not edit

extends Node

var score: int = 0

var high_score: int = 0

func add_score(points: int):
	score = score + points
	if score > high_score:
		high_score = score
	print(score)

func reset_score():
	score = 0
	print(0)

func _ready():
	add_score(10)
	add_score(25)
	add_score(50)
	add_score(15)
	print(high_score)
	reset_score()
	add_score(5)
	print(high_score)
