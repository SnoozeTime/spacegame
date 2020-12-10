const rust = import('./pkg');
var _ = require('lodash');

var canvas = document.createElement('canvas');
canvas.width = 1600;
canvas.height = 960;
canvas.id = 'luminance-canvas';

document.getElementsByTagName('body')[0].appendChild(canvas);

rust
  .then(wasm => {
    // get the scene and set it up (events)
    const scene = wasm.get_game('luminance-canvas');

    // handle the space (' ') key to toggle the tess method
    window.addEventListener('keydown', (event) => {

        wasm.create_audio_system(scene);
      switch (event.key) {
        case ' ':
          wasm.process_key(scene, wasm.VirtualKey.Space);
          break;
        case "Enter":
          wasm.process_key(scene, wasm.VirtualKey.Enter);
              break;
        case 'w':
          wasm.process_key(scene, wasm.VirtualKey.W);
          break;
        case 'a':
          wasm.process_key(scene, wasm.VirtualKey.A);
          break;
        case 'd':
          wasm.process_key(scene, wasm.VirtualKey.D);
          break;
        case 'q':
          wasm.process_key(scene, wasm.VirtualKey.Q);
          break;
        case 'e':
          wasm.process_key(scene, wasm.VirtualKey.E);
          break;
        case 'f':
          wasm.process_key(scene, wasm.VirtualKey.F);
          break;
        case 'ArrowLeft':
                      wasm.process_key(scene, wasm.VirtualKey.Left);
                      break;
                    case 'ArrowRight':
                      wasm.process_key(scene, wasm.VirtualKey.Right);
                      break;
        case 'Escape':
          wasm.process_key(scene, wasm.VirtualKey.Escape);
          break;
        default:
      }
    });


    window.addEventListener('keyup', (event) => {

        switch (event.key) {
            case ' ':
              wasm.release_key(scene, wasm.VirtualKey.Space);
              break;
            case "Enter":
              wasm.release_key(scene, wasm.VirtualKey.Enter);
                  break;
            case 'w':
              wasm.release_key(scene, wasm.VirtualKey.W);
              break;
            case 'a':
              wasm.release_key(scene, wasm.VirtualKey.A);
              break;
            case 'd':
              wasm.release_key(scene, wasm.VirtualKey.D);
              break;
            case 'q':
              wasm.release_key(scene, wasm.VirtualKey.Q);
              break;
            case 'e':
              wasm.release_key(scene, wasm.VirtualKey.E);
              break;
            case 'f':
              wasm.release_key(scene, wasm.VirtualKey.F);
              break;
            case 'ArrowLeft':
              wasm.release_key(scene, wasm.VirtualKey.Left);
              break;
            case 'ArrowRight':
              wasm.release_key(scene, wasm.VirtualKey.Right);
              break;
            case 'Escape':
              wasm.release_key(scene, wasm.VirtualKey.Escape);
              break;
                default:
        }
    });

    window.addEventListener('mousemove', _.throttle((evt) => {
        var rect = canvas.getBoundingClientRect();
        var x =(evt.clientX - rect.left) / (rect.right - rect.left) * canvas.width
        var y = (evt.clientY - rect.top) / (rect.bottom - rect.top) * canvas.height
        wasm.process_mouse_move(scene, x, y)
    }, 16), false);

    window.addEventListener('click', _.throttle((evt) => {
       wasm.process_mouse_click(scene)
    }, 16), false);

    setInterval(() => {
        wasm.render_scene(scene)
        wasm.prepare_input(scene);
    }, 16);
  })
  .catch(console.error);

