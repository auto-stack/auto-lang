# Auto-generated from player.at — do not edit

extends Area2D

signal hit

@export var speed: int = 400

var screen_size = Vector2.ZERO

func _ready():
	screen_size = get_viewport_rect().size
	hide()

func _process(delta: int):
	var velocity = Vector2.ZERO
	if Input.is_action_pressed("move_right"):
		velocity.x = velocity.x + 1
	if Input.is_action_pressed("move_left"):
		velocity.x = velocity.x - 1
	if Input.is_action_pressed("move_down"):
		velocity.y = velocity.y + 1
	if Input.is_action_pressed("move_up"):
		velocity.y = velocity.y - 1

	if velocity.length() > 0:
		velocity = velocity.normalized() * speed
		get_node("AnimatedSprite2D").play()
	else:
		get_node("AnimatedSprite2D").stop()

	position = position + velocity * delta
	position = position.clamp(Vector2.ZERO, screen_size)

	if velocity.x != 0:
		get_node("AnimatedSprite2D").animation = "right"
		get_node("AnimatedSprite2D").flip_v = false
		get_node("Trail").rotation = 0
		get_node("AnimatedSprite2D").flip_h = velocity.x < 0
	else:
		if velocity.y != 0:
			get_node("AnimatedSprite2D").animation = "up"
			if velocity.y > 0:
				rotation = PI
			else:
				rotation = 0

func start(pos: int):
	position = pos
	rotation = 0
	show()
	get_node("CollisionShape2D").disabled = false

func _on_body_entered(body: int):
	hide()
	hit.emit()
	get_node("CollisionShape2D").set_deferred("disabled", true)
