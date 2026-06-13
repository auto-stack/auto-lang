# Auto-generated from player.at — do not edit

extends CharacterBody2D

const WALK_FORCE: int = 600

const WALK_MAX_SPEED: int = 200

const STOP_FORCE: int = 1300

const JUMP_SPEED: int = 200

@onready var gravity = float(ProjectSettings.get_setting("physics/2d/default_gravity"))

func _physics_process(delta: float):

	var walk: int = WALK_FORCE * Input.get_axis("move_left", "move_right")

	if abs(walk) < WALK_FORCE * 0.2:
		velocity.x = move_toward(velocity.x, 0, STOP_FORCE * delta)
	else:
		velocity.x = velocity.x + walk * delta

	velocity.x = clamp(velocity.x, -WALK_MAX_SPEED, WALK_MAX_SPEED)


	velocity.y = velocity.y + gravity * delta


	move_and_slide()


	if is_on_floor():
		if Input.is_action_just_pressed("jump"):
			velocity.y = -JUMP_SPEED
