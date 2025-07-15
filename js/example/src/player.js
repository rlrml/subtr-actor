(function (window) {
    var cookieName = "ballchasing-replay-setting";
    window.Settings = function (defaults) {
        var settings;
        var val = readCookie(cookieName);
        var changed = false;
        if (val) {
            settings = JSON.parse(val);
            for (k in defaults) {
                if (!settings.hasOwnProperty(k)) {
                    settings[k] = defaults[k];
                    changed = true;
                }
            }
        } else {
            settings = defaults;
            changed = true;
        }
        if (changed) {
            var val = JSON.stringify(settings);
            createCookie(cookieName, val, 9999);
        }
        return {
            Get: function (k, v) {
                if (settings.hasOwnProperty(k)) {
                    return settings[k];
                }
                return v;
            },
            Set: function (k, v) {
                settings[k] = v;
                var val = JSON.stringify(settings);
                createCookie(cookieName, val, 9999);
            },
        };
    };
})(window);
function createBus() {
    var listeners = {};
    return {
        on: function (event, listener) {
            listeners[event] = listeners[event] || [];
            listeners[event].push(listener);
        },
        fire: function (event, data) {
            (listeners[event] || []).forEach(function (listener) {
                listener(data);
            });
        },
        fireLater: function (event, data) {
            var self = this;
            setTimeout(function () {
                self.fire(event, data);
            }, 0);
        },
    };
}
("use strict");
(function () {
    var uniforms = {
        texture: {
            value: new THREE.TextureLoader().load(
                "/static/textures/solid-particle.png",
            ),
        },
    };
    var shaderMaterial = new THREE.ShaderMaterial({
        uniforms: uniforms,
        vertexShader: document.getElementById("particle-vertex-shader")
            .textContent,
        fragmentShader: document.getElementById("particle-fragment-shader")
            .textContent,
        blending: THREE.NormalBlending,
        depthTest: false,
        transparent: true,
        vertexColors: true,
    });
    window.newTrail = function (options) {
        var positions = [],
            colors = [],
            sizes = [],
            ages = [],
            particleStart = [],
            particleEnd = [];
        for (var i = 0; i < options.count; i++) {
            positions.push(0, 0, 0);
            colors.push(options.color.r, options.color.g, options.color.b);
            ages.push(0);
            particleStart.push(0);
            particleEnd.push(0);
            sizes.push(options.size);
        }
        var trailGeometry = new THREE.BufferGeometry();
        trailGeometry.setAttribute(
            "position",
            new THREE.Float32BufferAttribute(positions, 3).setUsage(
                THREE.DynamicDrawUsage,
            ),
        );
        trailGeometry.setAttribute(
            "color",
            new THREE.Float32BufferAttribute(colors, 3),
        );
        trailGeometry.setAttribute(
            "size",
            new THREE.Float32BufferAttribute(sizes, 1),
        );
        trailGeometry.setAttribute(
            "age",
            new THREE.Float32BufferAttribute(ages, 1).setUsage(
                THREE.DynamicDrawUsage,
            ),
        );
        var trailModel = new THREE.Points(trailGeometry, shaderMaterial);
        var currentParticle = 0;
        function resetTrail() {
            for (var i = 0; i < options.count; i++) {
                trailModel.geometry.attributes.age.array[i] = 0;
                trailModel.geometry.attributes.position[3 * i] = 0;
                trailModel.geometry.attributes.position[3 * i + 1] = 0;
                trailModel.geometry.attributes.position[3 * i + 2] = 0;
                particleStart[i] = 0;
                particleEnd[i] = 0;
            }
            trailModel.geometry.attributes.position.needsUpdate = true;
            trailModel.geometry.attributes.age.needsUpdate = true;
            currentParticle = 0;
        }
        function spawn(time, posVec) {
            trailModel.geometry.attributes.age.array[currentParticle] =
                options.maxAlpha;
            trailModel.geometry.attributes.age.needsUpdate = true;
            var pos = trailModel.geometry.attributes.position.array;
            pos[currentParticle * 3] = posVec.x;
            pos[currentParticle * 3 + 1] = posVec.y;
            pos[currentParticle * 3 + 2] = posVec.z;
            trailModel.geometry.attributes.position.needsUpdate = true;
            particleStart[currentParticle] = time;
            particleEnd[currentParticle] = time + options.age;
            currentParticle++;
            if (currentParticle >= options.count) {
                currentParticle = 0;
            }
        }
        function ageTrail(time) {
            var ages = trailModel.geometry.attributes.age.array;
            for (var i = 0; i < ages.length; i++) {
                if (time < particleStart[i] || time >= particleEnd[i]) {
                    ages[i] = 0;
                } else {
                    if (time - particleStart[i] >= options.grace) {
                        ages[i] =
                            options.maxAlpha *
                            (1 -
                                (time - particleStart[i] - options.grace) /
                                    (particleEnd[i] -
                                        particleStart[i] -
                                        options.grace));
                    }
                }
            }
            trailModel.geometry.attributes.age.needsUpdate = true;
        }
        return {
            model: new THREE.Points(trailGeometry, shaderMaterial),
            age: ageTrail,
            spawn: spawn,
            reset: resetTrail,
        };
    };
    function dist(a, b) {
        var dx = a.x - b.x,
            dy = a.y - b.y,
            dz = a.z - b.z;
        return Math.sqrt(dx * dx + dy * dy + dz * dz);
    }
    function entityAtTime(t, entities) {
        for (var i = 0; i < entities.length; i++) {
            var res = entities[i];
            if (t >= res.start && t <= res.end) {
                return res;
            }
        }
    }
    function indexOfTimeInEntity(entity, t) {
        for (var i = 0; i < entity.times.length; i++) {
            if (entity.times[i] > t) {
                return i - 1;
            }
        }
        return entity.times.length - 1;
    }
    function posInTime(entity, time) {
        var tidx = indexOfTimeInEntity(entity, time);
        if (tidx === -1) {
            return;
        }
        var pidx = tidx * 3;
        var pos = {
            x: entity.pos[pidx],
            y: entity.pos[pidx + 1],
            z: entity.pos[pidx + 2],
        };
        if (tidx === entity.times.length - 1) {
            return pos;
        }
        var t0 = entity.times[tidx],
            t1 = entity.times[tidx + 1],
            ratio = (time - t0) / (t1 - t0);
        var f = function (idx) {
            return (
                entity.pos[idx] +
                ratio * (entity.pos[idx + 3] - entity.pos[idx])
            );
        };
        return { x: f(pidx), y: f(pidx + 1), z: f(pidx + 2) };
    }
    window.newTrailAnimator = function (entities, scene, bus, trailOptions) {
        trailOptions = Object.assign(
            {},
            {
                count: 100,
                color: { r: 0, g: 0, b: 0 },
                size: 10,
                maxAlpha: 0.5,
                age: 1,
                grace: 0,
                spawnDist: 20,
                enabled: false,
            },
            trailOptions,
        );
        var trail = window.newTrail(trailOptions);
        if (trailOptions.enabled) {
            scene.add(trail.model);
        }
        var animators = [];
        var currentTime,
            lastAgeTime,
            lastSpawnPos,
            lastSpawnTime = 0,
            lastEntity;
        function requestUpdate() {
            bus.fireLater("update");
        }
        function resetToTime(time) {
            trail.reset();
            var entity = entityAtTime(time, entities);
            if (!entity) {
                return;
            }
            lastEntity = entity;
            var firstTime = Math.max(0, time - trailOptions.age),
                timeStep = Math.max(
                    1 / 60,
                    (time - firstTime) / trailOptions.count,
                ),
                spawned = 0;
            lastSpawnTime = 0;
            lastSpawnPos = null;
            for (var t = time; t >= firstTime; t -= timeStep) {
                var pos = posInTime(entity, t);
                if (!pos) {
                    continue;
                }
                if (
                    lastSpawnPos == null ||
                    dist(pos, lastSpawnPos) > trailOptions.spawnDist
                ) {
                    trail.spawn(t, pos);
                    lastSpawnPos = pos;
                    spawned++;
                }
            }
            trail.age(time);
            requestUpdate();
        }
        bus.on("set-time", resetToTime);
        if (trailOptions.toggleEvent) {
            bus.on(trailOptions.toggleEvent, function (value) {
                if (value) {
                    trailOptions.enabled = true;
                    scene.add(trail.model);
                    resetToTime(currentTime);
                } else {
                    trailOptions.enabled = false;
                    scene.remove(trail.model);
                    requestUpdate();
                }
            });
        }
        if (trailOptions.durationEvent) {
            bus.on(trailOptions.durationEvent, function (value) {
                trailOptions.age = value;
                if (trailOptions.enabled) {
                    resetToTime(currentTime);
                }
            });
        }
        bus.on("trail-reset", function () {
            if (trailOptions.enabled) {
                trail.reset();
                requestUpdate();
            }
        });
        animators.push({
            anim: function (time) {
                currentTime = time;
                if (!trailOptions.enabled) {
                    return;
                }
                if (!lastAgeTime || time - lastAgeTime > 0.2) {
                    trail.age(time);
                    lastAgeTime = time;
                }
                var entity = entityAtTime(time, entities);
                if (!entity) {
                    trail.reset();
                    lastSpawnTime = 0;
                    lastSpawnPos = null;
                    lastEntity = null;
                    requestUpdate();
                    return;
                }
                if (entity !== lastEntity) {
                    trail.reset();
                    lastSpawnTime = 0;
                    lastSpawnPos = null;
                }
                lastEntity = entity;
                var timeStep = Math.max(
                    1 / 60,
                    trailOptions.age / trailOptions.count,
                );
                if (time - lastSpawnTime >= timeStep) {
                    var pos = posInTime(entity, time);
                    if (
                        pos &&
                        (lastSpawnPos == null ||
                            dist(pos, lastSpawnPos) > trailOptions.spawnDist)
                    ) {
                        trail.spawn(time, pos);
                        lastSpawnPos = pos;
                        lastSpawnTime = time;
                    }
                }
            },
        });
        return multiAnimator(animators);
    };
})();
function multiAnimator(animators) {
    return {
        anim: function (time) {
            animators.forEach(function (a) {
                a.anim(time);
            });
        },
    };
}
function nopAnimator() {
    return { anim: function () {} };
}
function animator(entities, model, scene, bus, options) {
    var enabled = options.enabled || true;
    var name = options.name || "unknown";
    var startTimes = [];
    var endTimes = [];
    options = options || {};
    if (options.dom) {
        options.css = options.css || {};
    }
    options.posShift = options.posShift || { x: 0, y: 0, z: 0 };
    entities.forEach(function (entity) {
        startTimes.push(entity.start);
        endTimes.push(entity.end);
    });
    var entityPointer = 0,
        entityAdded = false,
        entity,
        trackLen = 0;
    function addEntity() {
        if (entityAdded) {
            return;
        }
        if (options.dom) {
            model.appendTo("#player");
            model.css(options.css);
        } else {
            model.visible = true;
            scene.add(model);
        }
        entityAdded = true;
        entity = entities[entityPointer];
        trackLen = entity.times.length;
    }
    function removeEntity() {
        if (!entityAdded) {
            return;
        }
        if (options.dom) {
            model.remove();
        } else {
            scene.remove(model);
        }
        entityAdded = false;
    }
    bus.on("set-time", function (t) {
        for (var i = 0; i < startTimes.length; i++) {
            if (t >= startTimes[i] && t < endTimes[i]) {
                if (entityPointer !== i) {
                    removeEntity();
                }
                entityPointer = i;
                addEntity();
                return;
            }
        }
        entityPointer = 0;
        removeEntity();
    });
    if (options.toggleEvent) {
        bus.on(options.toggleEvent, function (value) {
            enabled = value;
            if (options.toggleEventInv) {
                enabled = !value;
            }
            if (entityAdded) {
                removeEntity();
                entityAdded = false;
            }
            bus.fireLater("update");
        });
    }
    return {
        anim: function (time) {
            if (!enabled) {
                return;
            }
            if (
                entityAdded &&
                endTimes[entityPointer] !== 0 &&
                time >= endTimes[entityPointer]
            ) {
                entityPointer++;
                removeEntity();
            }
            if (!entityAdded && time >= startTimes[entityPointer]) {
                addEntity();
            }
            if (!entityAdded) {
                return;
            }
            var stop = false,
                index;
            for (index = 0; !stop; ) {
                if (index === trackLen - 1) {
                    stop = true;
                    continue;
                }
                if (
                    time >= entity.times[index] &&
                    time <= entity.times[index + 1]
                ) {
                    stop = true;
                } else {
                    index++;
                }
            }
            var f = function (arr, idx) {
                    return arr[idx];
                },
                g = function (arr, idx) {
                    return arr[idx];
                };
            if (index < trackLen - 1) {
                var t0 = entity.times[index],
                    t1 = entity.times[index + 1],
                    ratio = (time - t0) / (t1 - t0);
                f = function (arr, idx) {
                    if (!arr || arr.length <= idx + 3) {
                        console.error("Invalid array or index in f function:", {arr: arr ? arr.length : 'undefined', idx});
                        return 0;
                    }
                    return arr[idx] + ratio * (arr[idx + 3] - arr[idx]);
                };
                if (options.rotation && !options.dom) {
                    g = function (arr, idx) {
                        if (!arr || arr.length <= idx + 3) {
                            console.error("Invalid array or index in g function:", {arr: arr ? arr.length : 'undefined', idx});
                            return 0;
                        }
                        if (Math.abs(arr[idx] - arr[idx + 3]) > Math.PI / 4) {
                            return arr[idx];
                        }
                        return arr[idx] + ratio * (arr[idx + 3] - arr[idx]);
                    };
                }
            }
            var pidx = 3 * index;
            if (options.dom) {
                var v = window.player.projectToScreen(
                        -f(entity.pos, pidx) + options.posShift.x,
                        f(entity.pos, pidx + 1) + options.posShift.y,
                        f(entity.pos, pidx + 2) + options.posShift.z,
                    ),
                    oldTop = options.css.top,
                    oldLeft = options.css.left;
                options.css.top = Math.floor(v.y) + "px";
                options.css.left = Math.floor(v.x) + "px";
                if (
                    oldTop !== options.css.top ||
                    oldLeft !== options.css.left
                ) {
                    model[0].style.left = options.css.left;
                    model[0].style.top = options.css.top;
                }
            } else {
                var newx = f(entity.pos, pidx) + options.posShift.x,
                    newy = f(entity.pos, pidx + 1) + options.posShift.y,
                    newz = f(entity.pos, pidx + 2) + options.posShift.z;
                if (options.posCallback) {
                    options.posCallback(time, { x: newx, y: newy, z: newz });
                } else {
                    model.position.set(newx, newy, newz);
                }
            }
            if (options.rotation && !options.dom) {
                var qidx = 4 * index;
                if (entity.quat && entity.quat.length > 0 && entity.quat.length > qidx + 3) {
                    model.quaternion.copy(
                        new THREE.Quaternion(
                            entity.quat[qidx] !== undefined ? entity.quat[qidx] : 0,
                            entity.quat[qidx + 1] !== undefined ? entity.quat[qidx + 1] : 0,
                            entity.quat[qidx + 2] !== undefined ? entity.quat[qidx + 2] : 0,
                            entity.quat[qidx + 3] !== undefined ? entity.quat[qidx + 3] : 1,
                        ),
                    );
                } else {
                    model.rotation.set(0, 0, 0);
                    if (entity.rot && entity.rot.length > pidx + 2) {
                        model.rotateZ(g(entity.rot, pidx + 2));
                        model.rotateX(g(entity.rot, pidx));
                        model.rotateY(g(entity.rot, pidx + 1));
                    }
                }
            }
        },
    };
}
(function () {
    function createSoccar(scene, mapCode) {
        var SOCCAR_YSIZE = 10280,
            SOCCAR_XSIZE = 8240,
            SOCCAR_DEPTH = 1960,
            STADIUM_CORNER = 1e3,
            GOAL_WIDTH = 1900,
            GOAL_HEIGHT = 800,
            GOAL_DEPTH = 900;
        if (mapCode === "labs_corridor_p") {
            SOCCAR_YSIZE = 14100;
            SOCCAR_XSIZE = 4080;
            SOCCAR_DEPTH = 1960;
            STADIUM_CORNER = 700;
            GOAL_WIDTH = 1800;
            GOAL_HEIGHT = 800;
            GOAL_DEPTH = 900;
        }
        function createBackWall(color, inverted) {
            var backWall = new THREE.Group();
            var opaqueMaterial = new THREE.MeshLambertMaterial({
                color: color,
            });
            var transparentMaterial = new THREE.MeshLambertMaterial({
                color: color,
                transparent: true,
                opacity: 0.6,
            });
            var sideWallWidth =
                SOCCAR_XSIZE / 2 - STADIUM_CORNER - GOAL_WIDTH / 2;
            var backWallLeft = new THREE.Mesh(
                new THREE.CubeGeometry(
                    sideWallWidth,
                    1,
                    SOCCAR_DEPTH,
                    20,
                    15,
                    1,
                ),
                inverted ? opaqueMaterial : transparentMaterial,
            );
            backWallLeft.position.set(
                sideWallWidth / 2 + GOAL_WIDTH / 2,
                0,
                SOCCAR_DEPTH / 2,
            );
            backWallLeft.castShadow = false;
            backWallLeft.receiveShadow = true;
            backWall.add(backWallLeft);
            var cornerWidth = Math.sqrt(2 * Math.pow(STADIUM_CORNER, 2));
            var backWallLeftCorner = new THREE.Mesh(
                new THREE.CubeGeometry(cornerWidth, 1, SOCCAR_DEPTH, 20, 15, 1),
                inverted ? opaqueMaterial : transparentMaterial,
            );
            backWallLeftCorner.position.set(
                SOCCAR_XSIZE / 2 - STADIUM_CORNER / 2,
                -STADIUM_CORNER / 2,
                SOCCAR_DEPTH / 2,
            );
            backWallLeftCorner.rotateZ(-Math.PI / 4);
            backWallLeftCorner.castShadow = false;
            backWallLeftCorner.receiveShadow = true;
            backWall.add(backWallLeftCorner);
            var backWallRight = new THREE.Mesh(
                new THREE.CubeGeometry(
                    sideWallWidth,
                    1,
                    SOCCAR_DEPTH,
                    20,
                    15,
                    1,
                ),
                transparentMaterial,
            );
            backWallRight.position.set(
                -sideWallWidth / 2 - GOAL_WIDTH / 2,
                0,
                SOCCAR_DEPTH / 2,
            );
            backWallRight.castShadow = false;
            backWallRight.receiveShadow = true;
            backWall.add(backWallRight);
            var backWallTop = new THREE.Mesh(
                new THREE.CubeGeometry(
                    GOAL_WIDTH,
                    1,
                    SOCCAR_DEPTH - GOAL_HEIGHT,
                    20,
                    15,
                    1,
                ),
                transparentMaterial,
            );
            backWallTop.position.set(0, 0, SOCCAR_DEPTH / 2 + GOAL_HEIGHT / 2);
            backWallTop.castShadow = false;
            backWallTop.receiveShadow = true;
            backWall.add(backWallTop);
            return backWall;
        }
        function createHalf(color, inverted) {
            var res = new THREE.Group();
            var floor = new THREE.Shape();
            floor.moveTo(SOCCAR_XSIZE / 2, 0);
            floor.lineTo(-SOCCAR_XSIZE / 2, 0);
            floor.lineTo(-SOCCAR_XSIZE / 2, SOCCAR_YSIZE / 2 - STADIUM_CORNER);
            floor.lineTo(-SOCCAR_XSIZE / 2 + STADIUM_CORNER, SOCCAR_YSIZE / 2);
            floor.lineTo(-GOAL_WIDTH / 2, SOCCAR_YSIZE / 2);
            floor.lineTo(-GOAL_WIDTH / 2, SOCCAR_YSIZE / 2 + GOAL_DEPTH);
            floor.lineTo(GOAL_WIDTH / 2, SOCCAR_YSIZE / 2 + GOAL_DEPTH);
            floor.lineTo(GOAL_WIDTH / 2, SOCCAR_YSIZE / 2);
            floor.lineTo(SOCCAR_XSIZE / 2 - STADIUM_CORNER, SOCCAR_YSIZE / 2);
            floor.lineTo(SOCCAR_XSIZE / 2, SOCCAR_YSIZE / 2 - STADIUM_CORNER);
            floor.lineTo(SOCCAR_XSIZE / 2, 0);
            var opaqueMaterial = new THREE.MeshLambertMaterial({
                color: color,
            });
            var transparentMaterial = new THREE.MeshLambertMaterial({
                color: color,
                transparent: true,
                opacity: 0.5,
            });
            var mesh = new THREE.Mesh(
                new THREE.ShapeGeometry(floor),
                opaqueMaterial,
            );
            mesh.receiveShadow = true;
            res.add(mesh);
            var goalFarPost = new THREE.Mesh(
                new THREE.PlaneBufferGeometry(GOAL_HEIGHT, GOAL_DEPTH, 6, 6),
                inverted ? transparentMaterial : opaqueMaterial,
            );
            goalFarPost.receiveShadow = true;
            goalFarPost.position.set(
                -GOAL_WIDTH / 2,
                SOCCAR_YSIZE / 2 + GOAL_DEPTH / 2,
                GOAL_HEIGHT / 2,
            );
            goalFarPost.rotateY(inverted ? -Math.PI / 2 : Math.PI / 2);
            res.add(goalFarPost);
            var goalNearPost = new THREE.Mesh(
                new THREE.PlaneBufferGeometry(GOAL_HEIGHT, GOAL_DEPTH, 6, 6),
                inverted ? opaqueMaterial : transparentMaterial,
            );
            goalNearPost.receiveShadow = true;
            goalNearPost.position.set(
                GOAL_WIDTH / 2,
                SOCCAR_YSIZE / 2 + GOAL_DEPTH / 2,
                GOAL_HEIGHT / 2,
            );
            goalNearPost.rotateY(inverted ? -Math.PI / 2 : Math.PI / 2);
            res.add(goalNearPost);
            var backWall = createBackWall(color, inverted);
            backWall.position.y = SOCCAR_YSIZE / 2;
            res.add(backWall);
            var sideWall1 = new THREE.Mesh(
                new THREE.PlaneBufferGeometry(
                    SOCCAR_DEPTH,
                    SOCCAR_YSIZE / 2 - STADIUM_CORNER,
                    6,
                    6,
                ),
                inverted ? opaqueMaterial : transparentMaterial,
            );
            sideWall1.position.set(
                SOCCAR_XSIZE / 2,
                (SOCCAR_YSIZE / 2 - STADIUM_CORNER) / 2,
                SOCCAR_DEPTH / 2,
            );
            sideWall1.rotateY(inverted ? -Math.PI / 2 : Math.PI / 2);
            sideWall1.castShadow = false;
            sideWall1.receiveShadow = true;
            res.add(sideWall1);
            var sideWall2 = new THREE.Mesh(
                new THREE.PlaneBufferGeometry(
                    SOCCAR_DEPTH,
                    SOCCAR_YSIZE / 2 - STADIUM_CORNER,
                    6,
                    6,
                ),
                inverted ? transparentMaterial : opaqueMaterial,
            );
            sideWall2.position.set(
                -SOCCAR_XSIZE / 2,
                (SOCCAR_YSIZE / 2 - STADIUM_CORNER) / 2,
                SOCCAR_DEPTH / 2,
            );
            sideWall2.rotateY(inverted ? -Math.PI / 2 : Math.PI / 2);
            sideWall2.castShadow = false;
            sideWall2.receiveShadow = true;
            res.add(sideWall2);
            return res;
        }
        var res = new THREE.Group();
        var orangePart = createHalf(16768115, false);
        res.add(orangePart);
        var bluePart = createHalf(8379391, true);
        bluePart.rotateZ(Math.PI);
        res.add(bluePart);
        scene.add(res);
        return {
            xsize: SOCCAR_XSIZE,
            ysize: SOCCAR_YSIZE,
            zsize: SOCCAR_DEPTH,
        };
    }
    function createHoops(scene) {
        var HOOPS_YSIZE = 7150,
            HOOPS_XSIZE = 5910,
            HOOPS_DEPTH = 1600,
            HOOPS_CORNER = 600;
        function createBackWall(color, inverted) {
            var backWall = new THREE.Group();
            var opaqueMaterial = new THREE.MeshLambertMaterial({
                color: color,
            });
            var transparentMaterial = new THREE.MeshLambertMaterial({
                color: color,
                transparent: true,
                opacity: 0.6,
            });
            var backWallWidth = HOOPS_XSIZE - 2 * HOOPS_CORNER;
            var backWallLeft = new THREE.Mesh(
                new THREE.CubeGeometry(
                    backWallWidth,
                    1,
                    HOOPS_DEPTH,
                    20,
                    15,
                    1,
                ),
                inverted ? opaqueMaterial : transparentMaterial,
            );
            backWallLeft.position.set(0, 0, HOOPS_DEPTH / 2);
            backWallLeft.castShadow = false;
            backWallLeft.receiveShadow = true;
            backWall.add(backWallLeft);
            var cornerWidth = Math.sqrt(2 * Math.pow(HOOPS_CORNER, 2));
            var backWallLeftCorner = new THREE.Mesh(
                new THREE.CubeGeometry(cornerWidth, 1, HOOPS_DEPTH, 20, 15, 1),
                inverted ? opaqueMaterial : transparentMaterial,
            );
            backWallLeftCorner.position.set(
                HOOPS_XSIZE / 2 - HOOPS_CORNER / 2,
                -HOOPS_CORNER / 2,
                HOOPS_DEPTH / 2,
            );
            backWallLeftCorner.rotateZ(-Math.PI / 4);
            backWallLeftCorner.castShadow = false;
            backWallLeftCorner.receiveShadow = true;
            backWall.add(backWallLeftCorner);
            return backWall;
        }
        function createHoop(team, inverted) {
            var HOOP_XSIZE = 1400,
                HOOP_YSIZE = 2400,
                HOOP_Z = 450;
            var xradius = HOOP_XSIZE / 2,
                yradius = HOOP_YSIZE / 2,
                segments = 20,
                oxradius = xradius + 60,
                oyradius = yradius + 60,
                bxradius = oxradius + 100,
                byradius = oyradius + 200,
                normal = new THREE.Vector3(0, 1, 0);
            var hoopGeometry = new THREE.Geometry(),
                vWallGeometry = new THREE.Geometry();
            for (var i = 0; i <= segments; i++) {
                var ang = (i * Math.PI) / segments,
                    xf = -Math.cos(ang),
                    yf = -Math.sin(ang);
                hoopGeometry.vertices.push(
                    new THREE.Vector3(xradius * xf, yradius * yf, HOOP_Z),
                    new THREE.Vector3(oxradius * xf, oyradius * yf, HOOP_Z),
                );
                vWallGeometry.vertices.push(
                    new THREE.Vector3(oxradius * xf, oyradius * yf, HOOP_Z),
                    new THREE.Vector3(bxradius * xf, byradius * yf, 0),
                );
                if (i > 0) {
                    hoopGeometry.faces.push(
                        new THREE.Face3(
                            2 * (i - 1),
                            2 * (i - 1) + 1,
                            2 * i + 1,
                            normal,
                        ),
                        new THREE.Face3(2 * (i - 1), 2 * i + 1, 2 * i, normal),
                    );
                    vWallGeometry.faces.push(
                        new THREE.Face3(
                            2 * (i - 1),
                            2 * (i - 1) + 1,
                            2 * i + 1,
                            normal,
                        ),
                        new THREE.Face3(2 * (i - 1), 2 * i + 1, 2 * i, normal),
                    );
                }
            }
            hoopGeometry.computeFaceNormals();
            hoopGeometry.computeVertexNormals();
            vWallGeometry.computeFaceNormals();
            vWallGeometry.computeVertexNormals();
            var darkColor = team === "blue" ? 2137326 : 15626072;
            var opaqueMaterial = new THREE.MeshLambertMaterial({
                color: darkColor,
            });
            var color = team === "blue" ? 8379391 : 16768115;
            var transparentMaterial = new THREE.MeshLambertMaterial({
                color: color,
                transparent: true,
                opacity: 0.5,
                side: THREE.DoubleSide,
            });
            var hoopMesh = new THREE.Mesh(hoopGeometry, opaqueMaterial);
            hoopMesh.castShadow = false;
            hoopMesh.receiveShadow = false;
            var vWallMesh = new THREE.Mesh(vWallGeometry, transparentMaterial);
            vWallMesh.castShadow = false;
            vWallMesh.receiveShadow = false;
            var res = new THREE.Group();
            res.add(hoopMesh);
            res.add(vWallMesh);
            return res;
        }
        function createHalf(team, inverted) {
            var color = team === "blue" ? 8379391 : 16768115;
            var res = new THREE.Group();
            var floor = new THREE.Shape();
            floor.moveTo(HOOPS_XSIZE / 2, 0);
            floor.lineTo(-HOOPS_XSIZE / 2, 0);
            floor.lineTo(-HOOPS_XSIZE / 2, HOOPS_YSIZE / 2 - HOOPS_CORNER);
            floor.lineTo(-HOOPS_XSIZE / 2 + HOOPS_CORNER, HOOPS_YSIZE / 2);
            floor.lineTo(HOOPS_XSIZE / 2 - HOOPS_CORNER, HOOPS_YSIZE / 2);
            floor.lineTo(HOOPS_XSIZE / 2, HOOPS_YSIZE / 2 - HOOPS_CORNER);
            floor.lineTo(HOOPS_XSIZE / 2, 0);
            var opaqueMaterial = new THREE.MeshLambertMaterial({
                color: color,
            });
            var transparentMaterial = new THREE.MeshLambertMaterial({
                color: color,
                transparent: true,
                opacity: 0.5,
            });
            var mesh = new THREE.Mesh(
                new THREE.ShapeGeometry(floor),
                opaqueMaterial,
            );
            mesh.receiveShadow = true;
            res.add(mesh);
            var backWall = createBackWall(color, inverted);
            backWall.position.y = HOOPS_YSIZE / 2;
            res.add(backWall);
            var sideWall1 = new THREE.Mesh(
                new THREE.PlaneBufferGeometry(
                    HOOPS_DEPTH,
                    HOOPS_YSIZE / 2 - HOOPS_CORNER,
                    6,
                    6,
                ),
                inverted ? opaqueMaterial : transparentMaterial,
            );
            sideWall1.position.set(
                HOOPS_XSIZE / 2,
                (HOOPS_YSIZE / 2 - HOOPS_CORNER) / 2,
                HOOPS_DEPTH / 2,
            );
            sideWall1.rotateY(inverted ? -Math.PI / 2 : Math.PI / 2);
            sideWall1.castShadow = false;
            sideWall1.receiveShadow = true;
            res.add(sideWall1);
            var sideWall2 = new THREE.Mesh(
                new THREE.PlaneBufferGeometry(
                    HOOPS_DEPTH,
                    HOOPS_YSIZE / 2 - HOOPS_CORNER,
                    6,
                    6,
                ),
                inverted ? transparentMaterial : opaqueMaterial,
            );
            sideWall2.position.set(
                -HOOPS_XSIZE / 2,
                (HOOPS_YSIZE / 2 - HOOPS_CORNER) / 2,
                HOOPS_DEPTH / 2,
            );
            sideWall2.rotateY(inverted ? -Math.PI / 2 : Math.PI / 2);
            sideWall2.castShadow = false;
            sideWall2.receiveShadow = true;
            res.add(sideWall2);
            var hoop = createHoop(team, inverted);
            hoop.position.y = HOOPS_YSIZE / 2;
            res.add(hoop);
            return res;
        }
        var res = new THREE.Group();
        var orangePart = createHalf("orange", false);
        res.add(orangePart);
        var bluePart = createHalf("blue", true);
        bluePart.rotateZ(Math.PI);
        res.add(bluePart);
        scene.add(res);
        return { xsize: HOOPS_XSIZE, ysize: HOOPS_YSIZE, zsize: HOOPS_DEPTH };
    }
    function addStadium(scene, mapCode, typ) {
        switch (typ) {
            case "soccar":
                return createSoccar(scene, mapCode);
            case "hoops":
                return createHoops(scene);
        }
    }
    window.Stadium = { add: addStadium };
})();
function createExplosion(scene, bus) {
    var TOTAL_OBJECTS = 2500,
        EXPLOSION_DURATION = 1,
        EXPLOSION_RADIUS = 1500,
        OBJECT_SIZE = 35,
        DISTANCE_TO_GOAL_LINE = 50,
        BLUE_EXPLOSION_COLOR = "#326bff",
        ORANGE_EXPLOSION_COLOR = "#ff9b02";
    var ballX,
        ballY,
        ballZ,
        explosionStartTime,
        explosion,
        dirs = [],
        material;
    function ExplodeAnimation(color) {
        var geometry = new THREE.Geometry();
        for (var i = 0; i < TOTAL_OBJECTS; i++) {
            var vertex = new THREE.Vector3();
            vertex.x = ballX;
            vertex.y = ballY;
            vertex.z = ballZ;
            geometry.vertices.push(vertex);
            var dirX = Math.random() * 2 * Math.PI;
            var dirY = Math.random() * 2 * Math.PI;
            dirs.push({
                x: Math.cos(dirY) * Math.cos(dirX),
                y: Math.sin(dirX),
                z: Math.sin(dirY) * Math.cos(dirX),
            });
        }
        material = new THREE.PointsMaterial({
            size: OBJECT_SIZE,
            color: color,
            transparent: true,
            opacity: 1,
        });
        this.object = new THREE.Points(geometry, material);
        scene.add(this.object);
    }
    bus.on("goal", function (p) {
        removeExplosion();
        explosionStartTime = p.time;
        ballX = ball.position.x;
        ballY =
            p.team === "blue"
                ? player.stadium.ysize / 2 - DISTANCE_TO_GOAL_LINE
                : -player.stadium.ysize / 2 + DISTANCE_TO_GOAL_LINE;
        ballZ = ball.position.z;
        var teamColor =
            p.team === "blue" ? BLUE_EXPLOSION_COLOR : ORANGE_EXPLOSION_COLOR;
        explosion = new ExplodeAnimation(teamColor);
    });
    function removeExplosion() {
        if (explosion != null) {
            scene.remove(explosion.object);
            explosion = null;
        }
    }
    return {
        anim: function (time) {
            if (!explosion) {
                return;
            }
            if (time < explosionStartTime) {
                removeExplosion();
                return;
            }
            var pCount = TOTAL_OBJECTS;
            var shifting =
                (Math.min(EXPLOSION_DURATION, time - explosionStartTime) /
                    EXPLOSION_DURATION) *
                EXPLOSION_RADIUS;
            var opacityShifting =
                (EXPLOSION_DURATION -
                    Math.min(EXPLOSION_DURATION, time - explosionStartTime)) /
                EXPLOSION_DURATION;
            while (pCount--) {
                var particle = explosion.object.geometry.vertices[pCount];
                if (particle.z <= 0) continue;
                particle.x = ballX + dirs[pCount].x * shifting;
                if (
                    particle.y <=
                        player.stadium.ysize / 2 - DISTANCE_TO_GOAL_LINE &&
                    particle.y >=
                        -player.stadium.ysize / 2 + DISTANCE_TO_GOAL_LINE
                ) {
                    particle.y = ballY + dirs[pCount].y * shifting;
                }
                particle.z = ballZ + dirs[pCount].z * shifting;
            }
            material.opacity = opacityShifting;
            explosion.object.geometry.verticesNeedUpdate = true;
            if (material.opacity <= 0) {
                removeExplosion();
            }
        },
    };
}
var ball;
function createBall(scene, bus, mapType, ballType) {
    if (!replayData.balls) {
        return nopAnimator();
    }
    var smult = 1,
        rot = false;
    if (mapType === "hoops") {
        smult = 1;
    } else if (ballType === "cube") {
        smult = 1.5;
        rot = true;
    }
    var RADIUS = 93 * smult;
    var SEGMENTS = 16;
    var RINGS = 16;
    ball =
        ballType === "cube"
            ? new THREE.Mesh(
                  new THREE.BoxGeometry(RADIUS, RADIUS, RADIUS),
                  new THREE.MeshLambertMaterial({ color: 16619048 }),
              )
            : new THREE.Mesh(
                  new THREE.SphereGeometry(RADIUS, SEGMENTS, RINGS),
                  new THREE.MeshLambertMaterial({ color: 16777215 }),
              );
    ball.position.set(0, 0, 0);
    ball.castShadow = true;
    ball.receiveShadow = false;
    var animators = [];
    animators.push(
        animator(replayData.balls, ball, scene, bus, {
            name: "ball",
            rotation: rot,
        }),
    );
    animators.push(
        window.newTrailAnimator(
            replayData.balls,
            scene,
            bus,
            {
                count: 1e3,
                color: { r: 0.5, g: 0.5, b: 0.5 },
                size: 2,
                maxAlpha: 1,
                age: 1,
                grace: 0,
                spawnDist: 5,
                toggleEvent: "setting.ball.trail",
                durationEvent: "setting.trail.duration",
            },
            {},
        ),
    );
    return multiAnimator(animators);
}
function createBoosts(scene, bus, options) {
    if (!replayData.boost_pads) {
        return nopAnimator();
    }
    var enabled = options.enabled || true;
    var f = 0.02,
        cmin = 0.4,
        cmax = 0.7,
        r = cmin,
        cfix = 0.1,
        prevTime = 0;
    var material = new THREE.MeshBasicMaterial({
        color: new THREE.Color(r, r, cfix),
    });
    function createPad(bp) {
        if (bp.big) {
            var r = 65;
            var geometry = new THREE.SphereBufferGeometry(r, 24, 24);
            var sphere = new THREE.Mesh(geometry, material);
            sphere.position.set(-bp.x, bp.y, r + 1);
            return sphere;
        } else {
            var r = 30;
            var geometry = new THREE.PlaneGeometry(r * 2, r * 2, 4);
            var circle = new THREE.Mesh(geometry, material);
            circle.position.set(-bp.x, bp.y, 1);
            circle.rotateZ(Math.PI / 4);
            return circle;
        }
    }
    var pads = [],
        added = {};
    for (var i = 0; i < replayData.boost_pads.length; i++) {
        var bp = replayData.boost_pads[i];
        pads.push(createPad(bp));
        added[i] = false;
    }
    if (options.toggleEvent) {
        bus.on(options.toggleEvent, function (value) {
            enabled = !value;
            for (var i = 0; i < pads.length; i++) {
                if (added[i]) {
                    scene.remove(pads[i]);
                    added[i] = false;
                }
            }
            bus.fireLater("update");
        });
    }
    function animPad(i, time) {
        var bp = replayData.boost_pads[i];
        if (!bp.events || !bp.events.start || !bp.events.end) {
            return;
        }
        for (var j = 0; j < bp.events.start.length; j++) {
            var s = bp.events.start[j],
                e = bp.events.end[j];
            if (s <= time && time <= e) {
                if (!added[i]) {
                    scene.add(pads[i]);
                    added[i] = true;
                }
                return true;
            }
        }
        return false;
    }
    bus.on("set-time", function (t) {
        prevTime = t;
    });
    return {
        anim: function (time) {
            if (!enabled) {
                return;
            }
            if (time - prevTime >= 0.1) {
                r += f;
                if (r > cmax) {
                    r = cmax;
                    f = -f;
                } else if (r < cmin) {
                    r = cmin;
                    f = -f;
                }
                for (var i = 0; i < replayData.boost_pads.length; i++) {
                    pads[i].material.color.setRGB(r, r, cfix);
                }
                prevTime = time;
            }
            for (var i = 0; i < replayData.boost_pads.length; i++) {
                if (!animPad(i, time) && added[i]) {
                    scene.remove(pads[i]);
                    added[i] = false;
                }
            }
        },
    };
}
(function () {
    if (!String.prototype.padStart) {
        String.prototype.padStart = function padStart(targetLength, padString) {
            targetLength = targetLength >> 0;
            padString = String(
                typeof padString !== "undefined" ? padString : " ",
            );
            if (this.length > targetLength) {
                return String(this);
            } else {
                targetLength = targetLength - this.length;
                if (targetLength > padString.length) {
                    padString += padString.repeat(
                        targetLength / padString.length,
                    );
                }
                return padString.slice(0, targetLength) + String(this);
            }
        };
    }
    var namePosShifts = {
        ortho: { x: 0, y: 0, z: 600 },
        top: { x: -400, y: 0, z: 1600 },
    };
    var boostPosShifts = {
        ortho: { x: 0, y: -500, z: 600 },
        top: { x: -500, y: -400, z: 1600 },
    };
    var lensMap = new THREE.TextureLoader().load(
        "/static/textures/lensflare0_alpha.png",
    );
    function createCar(carColor, fallbackColor, useFallbackColor, bus) {
        var color = carColor;
        if (useFallbackColor) {
            color = fallbackColor;
        }
        var res = new THREE.Group();
        var normal = new THREE.Vector3(0, 1, 0),
            ocol = new THREE.Color(color),
            bcol = new THREE.Color(0);
        var geometry = new THREE.Geometry();
        geometry.vertices.push(
            new THREE.Vector3(100, -100, 100),
            new THREE.Vector3(100, 100, 100),
            new THREE.Vector3(-100, 100, 100),
            new THREE.Vector3(-100, -100, 100),
            new THREE.Vector3(150, -220, 20),
            new THREE.Vector3(-150, -220, 20),
            new THREE.Vector3(130, -400, -20),
            new THREE.Vector3(-130, -400, -20),
            new THREE.Vector3(140, 170, 25),
            new THREE.Vector3(-140, 170, 25),
            new THREE.Vector3(130, 240, 25),
            new THREE.Vector3(-130, 240, 25),
            new THREE.Vector3(130, -400, -80),
            new THREE.Vector3(-130, -400, -80),
            new THREE.Vector3(150, -220, -80),
            new THREE.Vector3(-150, -220, -80),
            new THREE.Vector3(140, 170, -80),
            new THREE.Vector3(-140, 170, -80),
            new THREE.Vector3(130, 240, -80),
            new THREE.Vector3(-130, 240, -80),
        );
        geometry.faces.push(
            new THREE.Face3(0, 1, 2, normal, ocol),
            new THREE.Face3(0, 2, 3, normal, ocol),
            new THREE.Face3(4, 0, 5, normal, bcol),
            new THREE.Face3(0, 3, 5, normal, bcol),
            new THREE.Face3(6, 4, 5, normal, ocol),
            new THREE.Face3(6, 5, 7, normal, ocol),
            new THREE.Face3(1, 8, 9, normal, bcol),
            new THREE.Face3(1, 9, 2, normal, bcol),
            new THREE.Face3(4, 8, 1, normal, bcol),
            new THREE.Face3(4, 1, 0, normal, bcol),
            new THREE.Face3(3, 2, 9, normal, bcol),
            new THREE.Face3(3, 9, 5, normal, bcol),
            new THREE.Face3(8, 10, 11, normal, ocol),
            new THREE.Face3(8, 11, 9, normal, ocol),
            new THREE.Face3(12, 6, 7, normal, ocol),
            new THREE.Face3(12, 7, 13, normal, ocol),
            new THREE.Face3(7, 5, 15, normal, ocol),
            new THREE.Face3(7, 15, 13, normal, ocol),
            new THREE.Face3(6, 14, 4, normal, ocol),
            new THREE.Face3(12, 14, 6, normal, ocol),
            new THREE.Face3(14, 16, 4, normal, ocol),
            new THREE.Face3(4, 16, 8, normal, ocol),
            new THREE.Face3(5, 9, 15, normal, ocol),
            new THREE.Face3(15, 9, 17, normal, ocol),
            new THREE.Face3(16, 18, 8, normal, ocol),
            new THREE.Face3(8, 18, 10, normal, ocol),
            new THREE.Face3(9, 11, 17, normal, ocol),
            new THREE.Face3(17, 11, 19, normal, ocol),
            new THREE.Face3(10, 18, 11, normal, ocol),
            new THREE.Face3(11, 18, 19, normal, ocol),
            new THREE.Face3(14, 12, 13, normal, ocol),
            new THREE.Face3(14, 13, 15, normal, ocol),
            new THREE.Face3(16, 14, 15, normal, ocol),
            new THREE.Face3(16, 15, 17, normal, ocol),
            new THREE.Face3(18, 16, 17, normal, ocol),
            new THREE.Face3(18, 17, 19, normal, ocol),
        );
        geometry.computeFaceNormals();
        geometry.computeVertexNormals();
        bus.on("setting.cars.colors.simple", function (value) {
            var color = carColor;
            if (value) {
                color = fallbackColor;
            }
            for (var i = 0; i < geometry.faces.length; i++) {
                var face = geometry.faces[i];
                if (face.color == ocol) {
                    face.color.set(color);
                }
            }
            geometry.colorsNeedUpdate = true;
            bus.fireLater("update");
        });
        var bodyMesh = new THREE.Mesh(
            geometry,
            new THREE.MeshLambertMaterial({
                color: 16777215,
                vertexColors: THREE.FaceColors,
            }),
        );
        bodyMesh.castShadow = true;
        bodyMesh.receiveShadow = false;
        res.add(bodyMesh);
        var wheelMat = new THREE.MeshLambertMaterial({ color: 0 });
        function newWheel(x, y, z, w) {
            var wheel = new THREE.Mesh(
                new THREE.CylinderGeometry(70, 70, w, 10),
                wheelMat,
            );
            wheel.rotateZ(Math.PI / 2);
            wheel.position.set(x, y, z);
            return wheel;
        }
        res.add(newWheel(120, -300, -60, 50));
        res.add(newWheel(-120, -300, -60, 50));
        res.add(newWheel(120, 150, -60, 70));
        res.add(newWheel(-120, 150, -60, 70));
        var material = new THREE.MeshBasicMaterial({ map: lensMap });
        material.transparent = true;
        material.blending = THREE.NormalBlending;
        var spriteMaterial = new THREE.SpriteMaterial({
            map: lensMap,
            color: 16777215,
        });
        var boostIndicator = new THREE.Sprite(spriteMaterial);
        boostIndicator.scale.set(2e3, 2e3, 1);
        boostIndicator.position.set(0, 500, 0);
        res.add(boostIndicator);
        res.position.set(0, 0, 50);
        res.rotateZ(Math.PI / 2);
        var sc = 0.35;
        res.scale.set(sc, sc, sc);
        var res2 = new THREE.Group();
        res2.add(res);
        return { model: res2, boostIndicator: boostIndicator };
    }
    function addCars(scene, bus, mapType) {
        var namePosShift = { x: 0, y: 0, z: 0 },
            boostPosShift = { x: 0, y: 0, z: 0 };
        bus.on("camera-mode", function (mode) {
            var newShift = namePosShifts[mode] || { x: 0, y: 0, z: 0 };
            namePosShift.x = newShift.x;
            namePosShift.y = newShift.y;
            namePosShift.z = newShift.z;
            newShift = boostPosShifts[mode] || { x: 0, y: 0, z: 0 };
            boostPosShift.x = newShift.x;
            boostPosShift.y = newShift.y;
            boostPosShift.z = newShift.z;
        });
        var animators = [];
        function toCssColor(color) {
            var res = color.toString(16);
            return "#" + res.padStart(6, "0");
        }
        replayData.players.forEach(function (rplayer, rplayerIndex) {
            if (!rplayer.cars) {
                return;
            }
            var color = rplayer.color,
                scolor = rplayer.team === "blue" ? 2137326 : 15626072;
            var carRes = createCar(
                    color,
                    scolor,
                    window.player.settings.Get("cars.colors.simple"),
                    bus,
                ),
                carModel = carRes.model,
                boostIndicator = carRes.boostIndicator;
            if (mapType === "hoops") {
                carModel.scale.set(0.7, 0.7, 0.7);
            }
            animators.push(
                animator(rplayer.cars, carModel, scene, bus, {
                    name: "car." + rplayer.player,
                    rotation: true,
                }),
            );
            var carName = createText(rplayer.player, color);
            animators.push(
                animator(rplayer.cars, carName, scene, bus, {
                    name: "name." + rplayer.player,
                    posShift: namePosShift,
                    dom: true,
                    css: { color: toCssColor(color) },
                    rotation: false,
                    toggleEvent: "setting.cars.name.hide",
                    toggleEventInv: true,
                    enabled: !window.player.settings.Get("cars.name.hide"),
                }),
            );
            var carBoost = createText("-", "#fff");
            animators.push(
                animator(rplayer.cars, carBoost, scene, bus, {
                    name: "boost." + rplayer.player,
                    posShift: boostPosShift,
                    dom: true,
                    css: {
                        "text-align": "center",
                        width: "30px",
                        height: "30px",
                        "line-height": "30px",
                        "border-radius": "50%",
                        "font-weight": "bold",
                        "background-color": toCssColor(color),
                    },
                    rotation: false,
                    toggleEvent: "setting.cars.bam.hide",
                    toggleEventInv: true,
                    enabled: !window.player.settings.Get("cars.bam.hide"),
                }),
            );
            function randomAround(base, scale) {
                var r = Math.random() - 0.5;
                return base + 2 * scale * r;
            }
            if (rplayer.boost_amount.times && rplayer.boost_state.start) {
                var currentBoost,
                    boostIsOn = false,
                    boostIsOnTime,
                    boostIndicatorEnabled = !window.player.settings.Get(
                        "cars.boost.trail.hide",
                    );
                bus.on("setting.cars.boost.trail.hide", function (hide) {
                    boostIndicatorEnabled = !hide;
                    bus.fireLater("update");
                });
                animators.push({
                    anim: function (time) {
                        if (!boostIndicatorEnabled) {
                            boostIndicator.visible = false;
                            return;
                        }
                        var start = rplayer.boost_state.start,
                            end = rplayer.boost_state.end;
                        for (
                            var index = start.length - 1;
                            index >= 0;
                            index--
                        ) {
                            if (time >= start[index] && time < end[index]) {
                                if (!boostIsOn) {
                                    boostIsOn = true;
                                    boostIsOnTime = start[index];
                                    if (boostIndicatorEnabled) {
                                        boostIndicator.visible = true;
                                        boostIndicator.scale.set(
                                            randomAround(2e3, 500),
                                            randomAround(2e3, 500),
                                            1,
                                        );
                                    }
                                }
                                return;
                            }
                        }
                        boostIsOn = false;
                        boostIndicator.visible = false;
                    },
                });
                animators.push({
                    anim: function (time) {
                        var times = rplayer.boost_amount.times,
                            values = rplayer.boost_amount.values,
                            len = times.length;
                        for (var index = len - 1; index >= 0; index--) {
                            if (time >= times[index]) {
                                var b = values[index];
                                if (
                                    boostIsOn &&
                                    index < len - 2 &&
                                    values[index] > values[index + 1]
                                ) {
                                    var t0 = boostIsOnTime,
                                        t1 = times[index + 1],
                                        ratio = (time - t0) / (t1 - t0);
                                    b = Math.round(
                                        values[index] +
                                            ratio *
                                                (values[index + 1] -
                                                    values[index]),
                                    );
                                }
                                if (b !== currentBoost) {
                                    currentBoost = b;
                                    carBoost.text(b);
                                }
                                return;
                            }
                        }
                    },
                });
            }
            animators.push(
                window.newTrailAnimator(rplayer.cars, scene, bus, {
                    name: rplayer.player,
                    count: 1e3,
                    color: CarColors[rplayer.team].get(rplayerIndex),
                    size: 2,
                    maxAlpha: 1,
                    age: 1,
                    grace: 0,
                    spawnDist: 5,
                    toggleEvent: "setting.cars.trails",
                    durationEvent: "setting.trail.duration",
                }),
            );
            if (rplayer.tracks) {
                var shift = { x: 0, y: 0, z: -200 };
                for (var k in rplayer.tracks) {
                    if (rplayer.tracks.hasOwnProperty(k)) {
                        var track = rplayer.tracks[k];
                        if (!track.start) {
                            continue;
                        }
                        var trackDom = createText(k, track.color);
                        animators.push(
                            animator(rplayer.cars, trackDom, scene, bus, {
                                name: "track." + k + "." + rplayer.player,
                                posShift: $.extend({}, shift),
                                dom: true,
                                css: {
                                    "text-align": "center",
                                    "font-weight": "bold",
                                    background: "#999",
                                },
                                rotation: false,
                            }),
                        );
                        shift.z += 250;
                        var trackAnimator = {
                            k: k,
                            track: track,
                            trackDom: trackDom,
                            trackVisible: true,
                            anim: function (time) {
                                var starts = this.track.start,
                                    ends = this.track.end,
                                    len = starts.length;
                                for (var index = 0; index < len; index++) {
                                    if (
                                        time >= starts[index] &&
                                        time < ends[index]
                                    ) {
                                        if (this.trackVisible) {
                                            return;
                                        }
                                        this.trackVisible = true;
                                        this.trackDom.css({ opacity: 1 });
                                        return;
                                    }
                                }
                                if (this.trackVisible === true) {
                                    this.trackVisible = false;
                                    this.trackDom.css({ opacity: 0 });
                                }
                            },
                        };
                        animators.push(trackAnimator);
                    }
                }
            }
            if (rplayer.events) {
                if (rplayer.events.start) {
                    var eventDom = createText(k, "black");
                    animators.push(
                        animator(rplayer.cars, eventDom, scene, bus, {
                            name: "ev." + k + "." + rplayer.player,
                            posShift: { x: 0, y: 0, z: 200 },
                            dom: true,
                            css: {
                                "text-align": "center",
                                "font-weight": "bold",
                                background: "#fff",
                            },
                            rotation: false,
                        }),
                    );
                    var eventAnimator = {
                        k: k,
                        events: rplayer.events,
                        eventDom: eventDom,
                        anim: function (time) {
                            var starts = this.events.start,
                                ends = this.events.end,
                                texts = this.events.text,
                                len = starts.length,
                                content = [];
                            for (var index = 0; index < len; index++) {
                                if (
                                    time >= starts[index] &&
                                    time < ends[index]
                                ) {
                                    content.push(texts[index]);
                                }
                            }
                            if (content.length) {
                                this.eventDom.css({ opacity: 1 });
                                this.eventDom.text(content.join(" | "));
                            } else {
                                this.eventDom.css({ opacity: 0 });
                            }
                        },
                    };
                    animators.push(eventAnimator);
                }
            }
        });
        return multiAnimator(animators);
    }
    window.Cars = { add: addCars };
})();
("use strict");
(function () {
    function mkColors(colors) {
        return {
            get: function (idx) {
                return colors[idx % colors.length];
            },
        };
    }
    window.CarColors = {
        blue: mkColors([
            { r: 32 / 255, g: 156 / 255, b: 238 / 255 },
            { r: 21 / 255, g: 101 / 255, b: 192 / 255 },
            { r: 0 / 255, g: 121 / 255, b: 107 / 255 },
            { r: 56 / 255, g: 142 / 255, b: 60 / 255 },
            { r: 40 / 255, g: 53 / 255, b: 147 / 255 },
        ]),
        orange: mkColors([
            { r: 238 / 255, g: 111 / 255, b: 88 / 255 },
            { r: 192 / 255, g: 57 / 255, b: 43 / 255 },
            { r: 236 / 255, g: 64 / 255, b: 122 / 255 },
            { r: 142 / 255, g: 36 / 255, b: 170 / 255 },
            { r: 253 / 255, g: 216 / 255, b: 53 / 255 },
        ]),
    };
})();
function createCountdown(bus) {
    if (!replayData.countdowns) {
        return nopAnimator();
    }
    var times = replayData.countdowns,
        len = times.length,
        countdown = $("#countdown"),
        hasValue = false;
    return {
        anim: function (time) {
            for (var index = len - 1; index >= 0; index--) {
                if (time >= times[index] && time < times[index] + 5) {
                    var c = 3 - Math.floor(time - times[index]);
                    if (c === 0) {
                        c = "GO!";
                    }
                    hasValue = true;
                    if (c === -1) {
                        c = "";
                        hasValue = false;
                    }
                    countdown.text(c);
                    return;
                }
            }
            if (hasValue) {
                hasValue = false;
                countdown.text("");
            }
        },
    };
}
(function (window) {
    function handleSessionJoined(msg) {
        var watcherDom = $("#watcher-template").clone();
        watcherDom.removeAttr("id").attr("id", "watcher-" + msg.id);
        if (msg.user) {
            watcherDom
                .find("a")
                .attr("href", "/player/steam/" + msg.user["steam_id"]);
            watcherDom.find("span").text(msg.user["name"]);
            watcherDom.find("img").attr("src", msg.user["avatar"]);
        }
        watcherDom.show().appendTo(".watchers");
    }
    function handleSessionLeft(msg) {
        $("#watcher-" + msg.id).remove();
    }
    window.ReplayPlayer = function (options) {
        if (!window.replayData) {
            return;
        }
        var settings = options.settings;
        var slave = options.slave,
            master,
            bus = createBus(),
            player = {
                aspect: 1.4,
                bus: bus,
                playing: settings.Get("player.autoplay"),
                settings: settings,
            };
        window.player = player;
        function notify(action, data) {
            if (!master) {
                return;
            }
            data.action = action;
            master.send(JSON.stringify(data));
        }
        var container = document.querySelector("#player");
        var stats = new Stats();
        stats.showPanel(0);
        container.appendChild(stats.dom);
        var renderer = new THREE.WebGLRenderer();
        renderer.shadowMap.enabled = true;
        renderer.shadowMap.type = THREE.PCFSoftShadowMap;
        var scene = new THREE.Scene();
        scene.background = new THREE.Color(0);
        player.stadium = Stadium.add(
            scene,
            replayData["map"],
            replayData.map_type,
        );
        var camera = new THREE.OrthographicCamera();
        setCameraView(camera);
        window.player.projectToScreen = function (x, y, z) {
            var widthHalf = player.width / 2,
                heightHalf = player.height / 2;
            var pos = new THREE.Vector3(x, y, z);
            pos.project(camera);
            pos.x = pos.x * widthHalf + widthHalf;
            pos.y = -(pos.y * heightHalf) + heightHalf;
            return pos;
        };
        function cameraTop() {
            camera.up = new THREE.Vector3(0, 0, 1);
            camera.position.set(0, 0, player.stadium.zsize * 3);
            camera.lookAt(0, 0, 0);
        }
        function cameraSide() {
            camera.up = new THREE.Vector3(0, 0, 1);
            camera.position.set(
                player.stadium.zsize * 3 - player.stadium.zsize / 2,
                player.stadium.zsize * 3 - player.stadium.zsize / 2,
                player.stadium.zsize * 3,
            );
            camera.lookAt(
                -player.stadium.zsize / 2,
                -player.stadium.zsize / 2,
                0,
            );
        }
        var cameraModes = [
            { name: "ortho", apply: cameraSide },
            { name: "top", apply: cameraTop },
        ];
        var cameraModeIndex = 0;
        function applySelectedCamera() {
            player.bus.fire("camera-mode", cameraModes[cameraModeIndex].name);
            cameraModes[cameraModeIndex].apply();
            if (!player.playing) {
                update();
            }
            setCameraView(camera);
            notify("state", { camera: cameraModeIndex });
        }
        scene.add(camera);
        container.appendChild(renderer.domElement);
        function createLight() {
            var light = new THREE.DirectionalLight(16777215, 0.7);
            light.castShadow = true;
            light.position.set(0, 0, 2e3);
            light.shadow.mapSize.width = 1024;
            light.shadow.mapSize.height = 1024;
            light.shadow.camera.near = 100;
            light.shadow.camera.far = 3100;
            light.shadow.camera.left = -5e3;
            light.shadow.camera.right = 5e3;
            light.shadow.camera.top = -6e3;
            light.shadow.camera.bottom = 6e3;
            scene.add(light);
        }
        var ambient = new THREE.AmbientLight(6710886, 0.6);
        scene.add(ambient);
        createLight();
        var clock = new THREE.Clock();
        var time = 0;
        var initTime = location.hash.slice(1).match(/t=(.+)s/);
        if (initTime && initTime.length > 1) {
            var t0 = parseFloat(initTime[1]);
            if (t0 && t0 <= replayData.max_time) {
                time = t0;
            }
        }
        function persistTimeToHash(t) {
            if (slave) {
                return;
            }
            t = Math.floor(t * 100) / 100;
            var hash = location.hash;
            if (hash.includes("?")) {
                hash = hash.substr(0, hash.indexOf("?"));
            }
            hash += "?t=" + t + "s";
            location.hash = hash;
        }
        function setTime(t) {
            time = t;
            if (!player.playing) {
                setTimeout(update, 0);
            }
            persistTimeToHash(t);
            notify("state", { time: time });
        }
        player.bus.on("set-time", setTime);
        player.bus.on("update", function () {
            if (!player.playing) {
                update();
            }
        });
        var axisFix = new THREE.Group();
        var inv = new THREE.Matrix4().identity();
        inv.elements[0] = -1;
        axisFix.matrix.copy(inv);
        axisFix.matrixAutoUpdate = false;
        scene.add(axisFix);
        var boostsCtrl;
        boostsCtrl = createBoosts(scene, player.bus, {
            toggleEvent: "setting.boost.pads.hide",
            enabled: player.settings.Get("boost.pads.hide"),
        });
        var carsCtrl = Cars.add(axisFix, player.bus, replayData.map_type),
            ballCtrl = createBall(
                axisFix,
                player.bus,
                replayData.map_type,
                replayData.ball_type || "sphere",
            ),
            timeAndScore = createTimeAndScore(player.bus, options.slave),
            explosion = createExplosion(axisFix, player.bus),
            countdown = createCountdown(player.bus);
        function play() {
            player.playing = true;
            if (time > replayData.max_time) {
                player.bus.fire("set-time", 0);
            }
            clock.start();
            $("#play-pause")
                .find("i.fa")
                .removeClass("fa-play")
                .addClass("fa-pause");
            update();
            notify("state", { playing: true, time: time });
        }
        player.play = play;
        function pause() {
            player.playing = false;
            clock.stop();
            $("#play-pause")
                .find("i.fa")
                .removeClass("fa-pause")
                .addClass("fa-play");
            persistTimeToHash(time);
            notify("state", { playing: false, time: time });
        }
        player.pause = pause;
        function togglePlayback() {
            if (player.playing) {
                pause();
            } else {
                play();
            }
        }
        if (!slave) {
            $("#play-pause").on("click", togglePlayback);
        }
        if (!player.playing) {
            pause();
        }
        function toggleFullscreen() {
            if (player.fullscreen) {
                if (document.webkitCancelFullScreen) {
                    document.webkitCancelFullScreen();
                } else if (document.mozCancelFullScreen) {
                    document.mozCancelFullScreen();
                } else if (document.msCancelFullScreen) {
                    document.msCancelFullScreen();
                } else if (document.exitFullscreen) {
                    document.exitFullscreen();
                }
            } else {
                var elem = document.getElementById("player-container");
                if (elem.webkitRequestFullScreen) {
                    elem.webkitRequestFullScreen(Element.ALLOW_KEYBOARD_INPUT);
                } else if (elem.mozRequestFullScreen) {
                    elem.mozRequestFullScreen();
                } else if (elem.msRequestFullScreen) {
                    elem.msRequestFullScreen();
                } else if (elem.requestFullScreen) {
                    elem.requestFullScreen();
                }
            }
        }
        $("#full-screen").on("click", toggleFullscreen);
        function mkPopup(id, hotkey) {
            function openPopup() {
                $("#player-" + id + "-popup").addClass("is-active");
            }
            $("#player-" + id + "-btn").on("click", openPopup);
            function closePopup() {
                $("#player-" + id + "-popup").removeClass("is-active");
            }
            $("#player-" + id + "-popup").on(
                "click",
                ".modal-close",
                closePopup,
            );
            $("#player-" + id + "-popup").on("click", ".delete", closePopup);
            $("#player-" + id + "-popup").on(
                "click",
                ".modal-background",
                closePopup,
            );
            if (hotkey) {
                hotkeys(hotkey, openPopup);
            }
        }
        mkPopup("shortcuts", "/, shift+/");
        function hasFullSceenElement() {
            if (
                document.fullscreenElement ||
                document.webkitFullscreenElement ||
                document.mozFullscreenElement ||
                document.msFullscreenElement
            ) {
                return true;
            }
            return false;
        }
        $(document).on(
            "webkitfullscreenchange mozfullscreenchange fullscreenchange MSFullscreenChange",
            function () {
                player.fullscreen = hasFullSceenElement();
            },
        );
        if (!slave) {
            hotkeys("x", function () {
                togglePlayback();
            });
            hotkeys("left, shift+left, a, q", function (ev, handler) {
                var d = 1;
                if (handler.key.startsWith("shift+")) {
                    d = 10;
                }
                var v = Number($("#seekbar").val()) - d;
                if (v < 0) {
                    v = 0;
                }
                $("#seekbar").val(v.toString()).trigger("change");
            });
            hotkeys("right, shift+right, d", function (ev, handler) {
                var d = 1;
                if (handler.key.startsWith("shift+")) {
                    d = 10;
                }
                var v = Number($("#seekbar").val()) + d;
                $("#seekbar").val(v.toString()).trigger("change");
            });
            hotkeys("h", function (ev, handler) {
                $("#seekbar").val(0).trigger("change");
            });
            hotkeys("l", function (ev, handler) {
                $("#seekbar").val($("#seekbar").attr("max")).trigger("change");
            });
        }
        var speedMultipliers = [0.075, 0.125, 0.25, 0.5, 0.75, 1, 1.5, 2, 4, 8],
            speedMultiplierIndex = speedMultipliers.indexOf(1);
        function updateSpeedMultiplier() {
            notify("state", { speed: speedMultiplierIndex });
            $("#playback-speed-value").text(
                speedMultipliers[speedMultiplierIndex] + "x",
            );
        }
        function playbackSpeedUp() {
            if (speedMultiplierIndex === speedMultipliers.length - 1) {
                return;
            }
            speedMultiplierIndex++;
            updateSpeedMultiplier();
        }
        if (!slave) {
            hotkeys("w, z", playbackSpeedUp);
            $("#playback-speed-up").on("click", playbackSpeedUp);
        }
        function playbackSpeedDn() {
            if (speedMultiplierIndex === 0) {
                return;
            }
            speedMultiplierIndex--;
            updateSpeedMultiplier();
        }
        if (!slave) {
            hotkeys("s", playbackSpeedDn);
            $("#playback-speed-dn").on("click", playbackSpeedDn);
        }
        updateSpeedMultiplier();
        function toggleCamera() {
            cameraModeIndex++;
            if (cameraModeIndex >= cameraModes.length) {
                cameraModeIndex = 0;
            }
            applySelectedCamera();
            if (!player.playing) {
                update();
            }
        }
        if (!slave) {
            $("#cameraSwitcher").on("click", toggleCamera);
            hotkeys("c", toggleCamera);
        }
        function update() {
            stats.begin();
            if (player.playing) {
                time +=
                    speedMultipliers[speedMultiplierIndex] * clock.getDelta();
            }
            ballCtrl.anim(time);
            carsCtrl.anim(time);
            timeAndScore.anim(time);
            explosion.anim(time);
            countdown.anim(time);
            if (boostsCtrl) {
                boostsCtrl.anim(time);
            }
            renderer.render(scene, camera);
            stats.end();
            if (time > replayData.max_time) {
                pause();
                return;
            }
            if (player.playing) {
                requestAnimationFrame(update);
            }
        }
        function setCameraView(camera) {
            var widthInIsometricView =
                player.stadium.xsize * Math.cos(Math.PI / 4) +
                player.stadium.ysize * Math.cos(Math.PI / 4);
            var heightInIsometricView =
                player.stadium.xsize * Math.cos(Math.PI / 8) +
                player.stadium.zsize;
            var factor = 2.8;
            if (
                cameraModeIndex === 1 &&
                replayData["map"] === "labs_corridor_p"
            ) {
                factor = 2;
            }
            camera.left = (-widthInIsometricView / factor) * player.aspect;
            camera.right = (widthInIsometricView / factor) * player.aspect;
            camera.top = widthInIsometricView / factor;
            camera.bottom = -widthInIsometricView / factor;
            camera.near = 0;
            camera.far = 1e5;
            camera.updateProjectionMatrix();
        }
        function onWindowResize() {
            var playerDom = $("#player"),
                playerContainerDom = $("#player-container"),
                controlsHeight = 100,
                hudWidth = 200,
                countdownSize = 100;
            var width = $("#details-watch").width(),
                height = width / player.aspect,
                cheight = height + controlsHeight;
            var maxHeight = window.innerHeight - (player.fullscreen ? 0 : 180);
            if (cheight > maxHeight) {
                cheight = maxHeight;
                height = cheight - controlsHeight;
                width = height * player.aspect;
            }
            playerDom.css({
                "margin-left": ($("#details-watch").width() - width) / 2 + "px",
                width: width + "px",
                height: height + "px",
            });
            playerContainerDom.css({ height: cheight + "px" });
            player.width = width;
            player.height = height;
            setCameraView(camera);
            renderer.setSize(width, height);
            var gameInfoElement = document.getElementById("game-info");
            gameInfoElement.style.left = (width - hudWidth) / 2 + "px";
            var countdownElement = document.getElementById("countdown");
            countdownElement.style.left = (width - countdownSize) / 2 + "px";
            countdownElement.style.top = (height - countdownSize) / 2 + "px";
        }
        applySelectedCamera();
        update();
        onWindowResize();
        document.getElementById("player-container").style.display = "block";
        window.addEventListener("resize", onWindowResize, false);
        var currentSettings = {
            "cars.colors.simple": settings.Get("cars.colors.simple"),
            "cars.name.hide": settings.Get("cars.name.hide"),
            "cars.bam.hide": settings.Get("cars.bam.hide"),
            "boost.pads.hide": settings.Get("boost.pads.hide"),
            "cars.boost.trail.hide": settings.Get("cars.boost.trail.hide"),
        };
        [
            "cars.colors.simple",
            "cars.name.hide",
            "cars.bam.hide",
            "boost.pads.hide",
            "cars.boost.trail.hide",
            "cars.trails",
            "ball.trail",
            "trail.duration",
        ].forEach(function (s) {
            bus.on("setting." + s, function (v) {
                currentSettings[s] = v;
                notify("setting", { k: s, v: v });
            });
        });
        player.startMaster = function (ws) {
            master = ws;
            ws.addEventListener("open", function () {
                console.log("opened");
            });
            ws.addEventListener("close", function () {
                console.log("closed");
                $("#session-broken").addClass("is-active");
            });
            ws.addEventListener("message", function (e) {
                var msg = JSON.parse(e.data);
                console.log("got msg", msg);
                switch (msg.action) {
                    case "joined":
                        handleSessionJoined(msg);
                        notify("state", {
                            time: time,
                            camera: cameraModeIndex,
                            speed: speedMultiplierIndex,
                            playing: player.playing,
                            settings: currentSettings,
                        });
                        break;
                    case "left":
                        handleSessionLeft(msg);
                        break;
                }
            });
        };
        if (slave) {
            slave.addEventListener("close", function () {
                console.log("closed");
                $("#session-broken").addClass("is-active");
            });
            slave.addEventListener("message", function (e) {
                console.log("got msg", e.data);
                var msg = JSON.parse(e.data);
                switch (msg.action) {
                    case "ping":
                        break;
                    case "state":
                        if (msg.hasOwnProperty("time")) {
                            player.bus.fire("set-time", msg.time);
                        }
                        if (msg.hasOwnProperty("camera")) {
                            cameraModeIndex = msg.camera;
                            applySelectedCamera();
                        }
                        if (msg.hasOwnProperty("speed")) {
                            speedMultiplierIndex = msg.speed;
                            updateSpeedMultiplier();
                        }
                        if (msg.hasOwnProperty("playing")) {
                            if (msg.playing) {
                                play();
                            } else {
                                pause();
                            }
                        }
                        if (msg.hasOwnProperty("settings")) {
                            for (var k in msg.settings) {
                                if (msg.settings.hasOwnProperty(k)) {
                                    bus.fire("setting." + k, msg.settings[k]);
                                }
                            }
                        }
                        break;
                    case "setting":
                        bus.fire("setting." + msg.k, msg.v);
                        break;
                    case "quit":
                        player.pause();
                        slave.close();
                        $("#session-over").addClass("is-active");
                        break;
                    default:
                        console.error("unhandled action", msg.action);
                }
            });
        }
    };
})(window);
function createText(txt, color) {
    return $('<div class="txt"></div>').text(txt).css("color", color);
}
function createTimeAndScore(bus, slave) {
    var maxSeekBar = 1e3,
        maxTime = replayData.max_time,
        lastSeekUpdate = -1,
        seekbar = $("#seekbar");
    if (replayData.ticks) {
        replayData.ticks.forEach(function (tick) {
            var pos = (100 * tick.time) / maxTime;
            $('<span class="tick ' + tick.kind + '"></span>')
                .css("left", pos + "%")
                .appendTo("#player-controls .ticks");
        });
    }
    seekbar.attr("max", maxSeekBar);
    var remIndex = 0,
        remTime = [],
        remDom = document.getElementById("rem-seconds"),
        curTimeDom = document.getElementById("current-time"),
        totTimeDom = document.getElementById("total-time"),
        blueIndex = 0,
        orangeIndex = 0,
        blueDom = document.getElementById("blue-score"),
        orangeDom = document.getElementById("orange-score");
    totTimeDom.innerText = fmtTime(maxTime);
    function lpad(n) {
        if (n < 10) {
            return "0" + n.toString();
        }
        return n.toString();
    }
    function fmtTime(t) {
        var m = Math.floor(t / 60),
            s = Math.floor(t % 60);
        return lpad(m) + ":" + lpad(s);
    }
    var prevRemSecs = 300;
    for (var i = 0; i < replayData.rem_seconds.rem_seconds.length; i++) {
        var rs = replayData.rem_seconds.rem_seconds[i];
        remTime.push((rs > prevRemSecs ? "+" : "") + fmtTime(rs));
        prevRemSecs = rs;
    }
    var updateSeekbar = true;
    if (!slave) {
        seekbar.on("mousedown", function () {
            updateSeekbar = false;
        });
        seekbar.on("mouseup", function () {
            updateSeekbar = true;
        });
        seekbar.on("change", function () {
            var newTime = (seekbar.val() / maxSeekBar) * maxTime;
            bus.fire("set-time", newTime);
        });
    }
    function recomputeCountdown(t) {
        for (var i = 1; i <= replayData.rem_seconds.times.length; i++) {
            if (t < replayData.rem_seconds.times[i]) {
                remDom.innerText = remTime[i - 1];
                remIndex = i;
                return;
            }
        }
        remDom.innerText = remTime[0];
        remIndex = 1;
    }
    function recomputeScore(t, score, dom) {
        if (!score || !score.times) {
            return 0;
        }
        for (var i = score.times.length - 1; i >= 0; i--) {
            if (t > score.times[i]) {
                dom.innerText = score.score[i].toString();
                return i + 1;
            }
        }
        dom.innerText = "0";
        return 0;
    }
    bus.on("set-time", function (t) {
        lastSeekUpdate = -1;
        recomputeCountdown(t);
        blueIndex = recomputeScore(t, replayData.blue_score, blueDom);
        orangeIndex = recomputeScore(t, replayData.orange_score, orangeDom);
    });
    return {
        anim: function (time) {
            if (updateSeekbar && time - lastSeekUpdate > 0.5) {
                lastSeekUpdate = time;
                seekbar.val(Math.floor((maxSeekBar * time) / maxTime));
                curTimeDom.innerText = fmtTime(time);
            }
            var times = replayData.rem_seconds.times;
            if (remIndex < times.length) {
                if (time >= times[remIndex]) {
                    remDom.innerText = remTime[remIndex];
                    remIndex++;
                }
            }
            if (replayData.blue_score.times) {
                var blueScore = replayData.blue_score;
                if (blueIndex < blueScore.times.length) {
                    if (time >= blueScore.times[blueIndex]) {
                        blueDom.innerText =
                            blueScore.score[blueIndex].toString();
                        blueIndex++;
                        bus.fire("goal", { team: "blue", time: time });
                    }
                }
            }
            if (replayData.orange_score.times) {
                var orangeScore = replayData.orange_score;
                if (orangeIndex < orangeScore.times.length) {
                    if (time >= orangeScore.times[orangeIndex]) {
                        orangeDom.innerText =
                            orangeScore.score[orangeIndex].toString();
                        orangeIndex++;
                        bus.fire("goal", { team: "orange", time: time });
                    }
                }
            }
        },
    };
}
