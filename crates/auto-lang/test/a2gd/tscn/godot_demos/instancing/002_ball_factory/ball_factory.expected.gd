# Auto-generated from ball_factory.at — do not edit

extends Node2D

@export var ball_scene: PackedScene = preload("res://ball.tscn")

func _unhandled_input(input_event: InputEvent):
	if input_event.is_echo():
		return null
	if input_event.is_class("InputEventMouseButton"):
		if input_event.is_pressed():
			if input_event.button_index == MOUSE_BUTTON_LEFT:
				spawn_ball(get_global_mouse_position())

func spawn_ball(spawn_global_position: Vector2):
	var instance = ball_scene.instantiate()
	instance.global_position = spawn_global_position
	add_child(instance)
