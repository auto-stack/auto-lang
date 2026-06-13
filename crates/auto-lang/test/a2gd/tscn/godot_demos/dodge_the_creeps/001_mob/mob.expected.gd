# Auto-generated from mob.at — do not edit

extends RigidBody2D

func _ready():
	var mob_types = Array(get_node("AnimatedSprite2D").sprite_frames.get_animation_names())
	get_node("AnimatedSprite2D").animation = mob_types.pick_random()
	get_node("AnimatedSprite2D").play()

func _on_VisibilityNotifier2D_screen_exited():
	queue_free()
