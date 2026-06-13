# Auto-generated from troll.at — do not edit

extends CharacterBody2D

const MOTION_SPEED: int = 30

const FRICTION_FACTOR: float = 0.89

const TAN30DEG = tan(deg_to_rad(30))

func _physics_process(delta: float):
	var motion = Vector2()
	motion.x = Input.get_axis("move_left", "move_right")
	motion.y = Input.get_axis("move_up", "move_down")

	motion.y = motion.y * TAN30DEG
	velocity = velocity + motion.normalized() * MOTION_SPEED

	velocity = velocity * FRICTION_FACTOR
	move_and_slide()
