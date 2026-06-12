import * as THREE from 'three';

export class InputManager {
    constructor(camera, domElement) {
        this.camera = camera;
        this.moveState = {
            forward: false,
            backward: false,
            left: false,
            right: false,
            up: false,
            down: false
        };
        this.velocity = new THREE.Vector3();
        this.direction = new THREE.Vector3();
        this.moveSpeed = 25;

        this.setupEventListeners(domElement);
    }

    setupEventListeners(domElement) {
        // Keyboard only - pointer lock removed in favor of right-click drag in CameraManager
        document.addEventListener('keydown', (e) => this.onKeyDown(e));
        document.addEventListener('keyup', (e) => this.onKeyUp(e));
    }

    onKeyDown(event) {
        // Ignore keyboard input when user is typing in an input field
        const target = event.target;
        if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable) {
            return;
        }

        switch (event.code) {
            case 'ArrowUp':
            case 'KeyW': this.moveState.forward = true; break;
            case 'ArrowLeft':
            case 'KeyA': this.moveState.left = true; break;
            case 'ArrowDown':
            case 'KeyS': this.moveState.backward = true; break;
            case 'ArrowRight':
            case 'KeyD': this.moveState.right = true; break;
            case 'Space': this.moveState.up = true; break;
            case 'ShiftLeft':
            case 'ShiftRight': this.moveState.down = true; break;
        }
    }

    onKeyUp(event) {
        // Always process keyup to prevent stuck keys when focus changes
        switch (event.code) {
            case 'ArrowUp':
            case 'KeyW': this.moveState.forward = false; break;
            case 'ArrowLeft':
            case 'KeyA': this.moveState.left = false; break;
            case 'ArrowDown':
            case 'KeyS': this.moveState.backward = false; break;
            case 'ArrowRight':
            case 'KeyD': this.moveState.right = false; break;
            case 'Space': this.moveState.up = false; break;
            case 'ShiftLeft':
            case 'ShiftRight': this.moveState.down = false; break;
        }
    }

    update(delta) {
        // Movement is now handled by CameraManager
        // This method is kept for backwards compatibility
    }

    setCameraPosition(position, lookAt) {
        this.camera.position.copy(position);
        if (lookAt) {
            this.camera.lookAt(lookAt);
        }
    }
}
