# Auto-generated from annot.at — do not edit

extends Node

@export_range(0, 100, 1) var hp: int = 50

@onready var sprite = get_node("Sprite")

@export_group("Combat") var damage: int = 10

@export var speed: float = 300

func _ready():
	print(hp)
