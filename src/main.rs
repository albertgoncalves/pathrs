mod defer;
mod ffi;
mod math;

use crate::defer::Defer;
use crate::math::Normalize;
use std::convert::TryInto;
use std::ffi::{c_char, c_int, c_void, CStr, CString};
use std::fs::read_to_string;
use std::mem;
use std::path::Path;
use std::ptr;
use std::slice::from_raw_parts;
use std::str::from_utf8_unchecked;
use std::time;

#[repr(C)]
struct Geom<T> {
    translate: math::Vec2<T>,
    scale: math::Vec2<T>,
    color: math::Vec3<T>,
}

extern "C" fn callback_glfw_error(error_code: c_int, description: *const c_char) {
    let mut message = error_code.to_string();
    if !description.is_null() {
        message
            .push_str(&format!(": {}", unsafe { CStr::from_ptr(description) }.to_str().unwrap()));
    }
    panic!("{}", message);
}

extern "C" fn callback_glfw_key(
    window: *mut ffi::GLFWwindow,
    key: c_int,
    _scancode: c_int,
    action: c_int,
    _mods: c_int,
) {
    if action != ffi::GLFW_PRESS {
        return;
    }
    if key == ffi::GLFW_KEY_ESCAPE {
        unsafe {
            ffi::glfwSetWindowShouldClose(window, 1);
        }
    }
}

extern "C" fn callback_gl_debug(
    _source: ffi::GLenum,
    _type: ffi::GLenum,
    _id: ffi::GLuint,
    _severity: ffi::GLenum,
    length: ffi::GLsizei,
    message: *const ffi::GLchar,
    #[allow(non_snake_case)] _userParam: *const c_void,
) {
    assert!(0 < length);
    let message: &str = unsafe {
        from_utf8_unchecked(from_raw_parts(message.cast::<u8>(), length.try_into().unwrap()))
    };
    panic!("{}", message);
}

fn compile_shader(r#type: ffi::GLenum, source: &str) -> ffi::GLuint {
    unsafe {
        let shader = ffi::glCreateShader(r#type);
        ffi::glShaderSource(
            shader,
            1,
            [source.as_bytes().as_ptr().cast::<ffi::GLchar>()].as_ptr(),
            [source.as_bytes().len().try_into().unwrap()].as_ptr(),
        );
        ffi::glCompileShader(shader);
        shader
    }
}

macro_rules! buffer {
    ($target:expr, $ty:ty, $data:expr, $usage:expr $(,)?) => {
        ffi::glBindBuffer(ffi::GL_ARRAY_BUFFER, $target);
        ffi::glBufferData(
            ffi::GL_ARRAY_BUFFER,
            (mem::size_of::<$ty>() * $data.len()).try_into().unwrap(),
            $data.as_ptr().cast::<c_void>(),
            $usage,
        );
    };
}

macro_rules! size_of_field {
    ($ty:ty, $field:ident $(,)?) => {{
        const fn infer<T>(_: *const T) -> usize {
            mem::size_of::<T>()
        }
        let r#struct = mem::MaybeUninit::<$ty>::uninit();
        let field = ptr::addr_of!((*r#struct.as_ptr()).$field);
        infer(field)
    }};
}

macro_rules! attribute {
    ($program:expr, $ty:ty, $ident:ident $(,)?) => {{
        let index = ffi::glGetAttribLocation(
            $program,
            CStr::from_bytes_with_nul(concat!(stringify!($ident), '\0').as_bytes())
                .unwrap()
                .as_ptr()
                .cast::<ffi::GLchar>(),
        )
        .try_into()
        .unwrap();
        ffi::glEnableVertexAttribArray(index);
        ffi::glVertexAttribPointer(
            index,
            (mem::size_of::<$ty>() / mem::size_of::<ffi::GLfloat>())
                .try_into()
                .unwrap(),
            ffi::GL_FLOAT,
            ffi::GL_FALSE,
            (mem::size_of::<$ty>()).try_into().unwrap(),
            ptr::null::<c_void>(),
        );
    }};
    ($program:expr, $ty:ty, $field:ident, $div:expr $(,)?) => {{
        let index = ffi::glGetAttribLocation(
            $program,
            CStr::from_bytes_with_nul(concat!(stringify!($field), '\0').as_bytes())
                .unwrap()
                .as_ptr()
                .cast::<ffi::GLchar>(),
        )
        .try_into()
        .unwrap();
        ffi::glEnableVertexAttribArray(index);
        ffi::glVertexAttribPointer(
            index,
            (size_of_field!($ty, $field) / mem::size_of::<ffi::GLfloat>())
                .try_into()
                .unwrap(),
            ffi::GL_FLOAT,
            ffi::GL_FALSE,
            (mem::size_of::<$ty>()).try_into().unwrap(),
            mem::offset_of!($ty, $field) as *const c_void,
        );
        ffi::glVertexAttribDivisor(index, $div);
    }};
}

fn main() {
    unsafe {
        println!(
            "{}",
            CStr::from_ptr(ffi::glfwGetVersionString())
                .to_str()
                .unwrap(),
        );

        ffi::glfwSetErrorCallback(callback_glfw_error);

        assert!(ffi::glfwInit() == 1);
        defer!(ffi::glfwTerminate());

        ffi::glfwWindowHint(ffi::GLFW_OPENGL_DEBUG_CONTEXT, 1);
        ffi::glfwWindowHint(ffi::GLFW_CONTEXT_VERSION_MAJOR, 3);
        ffi::glfwWindowHint(ffi::GLFW_CONTEXT_VERSION_MINOR, 3);
        ffi::glfwWindowHint(ffi::GLFW_OPENGL_PROFILE, ffi::GLFW_OPENGL_CORE_PROFILE);
        ffi::glfwWindowHint(ffi::GLFW_RESIZABLE, 0);
        ffi::glfwWindowHint(ffi::GLFW_SAMPLES, 16);

        let width = 1400;
        let height = 900;

        let window = ffi::glfwCreateWindow(
            width,
            height,
            CString::new(std::module_path!())
                .unwrap()
                .as_bytes_with_nul()
                .as_ptr()
                .cast::<c_char>(),
            ptr::null_mut::<ffi::GLFWmonitor>(),
            ptr::null_mut::<ffi::GLFWwindow>(),
        );

        assert!(!window.is_null());
        defer!(ffi::glfwDestroyWindow(window));

        ffi::glfwMakeContextCurrent(window);
        ffi::glfwSwapInterval(1);
        ffi::glfwSetKeyCallback(window, callback_glfw_key);

        ffi::glEnable(ffi::GL_DEBUG_OUTPUT);
        ffi::glEnable(ffi::GL_DEBUG_OUTPUT_SYNCHRONOUS);
        ffi::glDebugMessageCallback(callback_gl_debug, ptr::null::<c_void>());

        ffi::glEnable(ffi::GL_BLEND);
        ffi::glBlendFunc(ffi::GL_SRC_ALPHA, ffi::GL_ONE_MINUS_SRC_ALPHA);
        ffi::glClearColor(0.1, 0.1, 0.1, 1.0);
        ffi::glEnable(ffi::GL_MULTISAMPLE);
        ffi::glViewport(0, 0, width, height);

        let program = ffi::glCreateProgram();
        {
            let vert_shader = compile_shader(
                ffi::GL_VERTEX_SHADER,
                &read_to_string(Path::new("src").join("vert.glsl")).unwrap(),
            );
            defer!(ffi::glDeleteShader(vert_shader));

            let frag_shader = compile_shader(
                ffi::GL_FRAGMENT_SHADER,
                &read_to_string(Path::new("src").join("frag.glsl")).unwrap(),
            );
            defer!(ffi::glDeleteShader(frag_shader));

            ffi::glAttachShader(program, vert_shader);
            ffi::glAttachShader(program, frag_shader);
            ffi::glLinkProgram(program);
        }
        ffi::glUseProgram(program);

        let vertices: [math::Vec2<ffi::GLfloat>; 4] = [
            math::Vec2 { x: 0.5, y: 0.5 },
            math::Vec2 { x: 0.5, y: -0.5 },
            math::Vec2 { x: -0.5, y: 0.5 },
            math::Vec2 { x: -0.5, y: -0.5 },
        ];

        let mut geoms: [Geom<ffi::GLfloat>; 1] = [Geom {
            translate: math::Vec2::default(),
            scale: 25.0.into(),
            color: math::Vec3 {
                x: 1.0,
                y: 0.5,
                z: 0.75,
            },
        }];

        let mut vao: ffi::GLuint = 0;
        ffi::glGenVertexArrays(1, &mut vao);
        defer!(ffi::glDeleteVertexArrays(1, &vao));

        let mut vbo: ffi::GLuint = 0;
        ffi::glGenBuffers(1, &mut vbo);
        defer!(ffi::glDeleteBuffers(1, &vbo));

        let mut instance_vbo: ffi::GLuint = 0;
        ffi::glGenBuffers(1, &mut instance_vbo);
        defer!(ffi::glDeleteBuffers(1, &instance_vbo));

        ffi::glBindVertexArray(vao);

        buffer!(vbo, math::Vec2<ffi::GLfloat>, vertices, ffi::GL_STATIC_DRAW);
        attribute!(program, math::Vec2<ffi::GLfloat>, position);

        buffer!(instance_vbo, Geom<ffi::GLfloat>, geoms, ffi::GL_DYNAMIC_DRAW);
        attribute!(program, Geom<ffi::GLfloat>, translate, 1);
        attribute!(program, Geom<ffi::GLfloat>, scale, 1);
        attribute!(program, Geom<ffi::GLfloat>, color, 1);

        #[allow(clippy::cast_precision_loss)]
        let projection = math::orthographic(0.0, width as f32, height as f32, 0.0, -1.0, 1.0);

        #[allow(clippy::cast_precision_loss)]
        let view = math::translate_rotate(
            math::Vec2 {
                x: (width as f32) / 2.0,
                y: (height as f32) / 2.0,
            },
            0.0,
        );

        ffi::glUniformMatrix4fv(
            ffi::glGetUniformLocation(program, c"projection".as_ptr().cast::<ffi::GLchar>()),
            1,
            ffi::GL_FALSE,
            projection.as_ptr().cast::<ffi::GLfloat>(),
        );
        ffi::glUniformMatrix4fv(
            ffi::glGetUniformLocation(program, c"view".as_ptr().cast::<ffi::GLchar>()),
            1,
            ffi::GL_FALSE,
            view.as_ptr().cast::<ffi::GLfloat>(),
        );

        let mut now = time::Instant::now();
        let mut frames = 0;

        let vertices_len = vertices.len().try_into().unwrap();
        let geoms_size = (mem::size_of::<Geom<ffi::GLfloat>>() * geoms.len())
            .try_into()
            .unwrap();
        let geoms_len = geoms.len().try_into().unwrap();

        let mut speed: math::Vec2<f32> = math::Vec2::default();

        let run: math::Vec2<f32> = 3.5.into();
        let drag: math::Vec2<f32> = 0.8125.into();

        println!("\n");
        while ffi::glfwWindowShouldClose(window) != 1 {
            if 0 < now.elapsed().as_secs() {
                println!(
                    "\x1B[2A\
                     {:10} ns/f\n\
                     {frames:10} frames",
                    now.elapsed().as_nanos() / frames,
                );
                now = time::Instant::now();
                frames = 0;
            }

            ffi::glfwPollEvents();

            let mut r#move: math::Vec2<f32> = math::Vec2::default();
            if ffi::glfwGetKey(window, ffi::GLFW_KEY_W) == ffi::GLFW_PRESS {
                r#move.y -= 1.0;
            }
            if ffi::glfwGetKey(window, ffi::GLFW_KEY_S) == ffi::GLFW_PRESS {
                r#move.y += 1.0;
            }
            if ffi::glfwGetKey(window, ffi::GLFW_KEY_A) == ffi::GLFW_PRESS {
                r#move.x -= 1.0;
            }
            if ffi::glfwGetKey(window, ffi::GLFW_KEY_D) == ffi::GLFW_PRESS {
                r#move.x += 1.0;
            }
            r#move.normalize();

            speed += r#move * run;
            speed *= drag;
            geoms[0].translate += speed;

            ffi::glClear(ffi::GL_COLOR_BUFFER_BIT);

            ffi::glBufferSubData(
                ffi::GL_ARRAY_BUFFER,
                0,
                geoms_size,
                geoms.as_ptr().cast::<c_void>(),
            );
            ffi::glDrawArraysInstanced(ffi::GL_TRIANGLE_STRIP, 0, vertices_len, geoms_len);

            ffi::glfwSwapBuffers(window);

            frames += 1;
        }
    }
}
