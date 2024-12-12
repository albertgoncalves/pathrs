use std::ffi::{c_char, c_double, c_float, c_int, c_long, c_uchar, c_uint, c_void};
use std::marker;

macro_rules! opaque_struct {
    ($name:ident) => {
        // NOTE: See `https://doc.rust-lang.org/nomicon/ffi.html#representing-opaque-structs`.
        #[repr(C)]
        pub struct $name {
            _data: [u8; 0],
            _marker: marker::PhantomData<(*mut u8, marker::PhantomPinned)>,
        }
    };
}

opaque_struct!(GLFWwindow);
opaque_struct!(GLFWmonitor);

pub type GLenum = c_uint;
pub type GLbitfield = c_uint;
pub type GLint = c_int;
pub type GLuint = c_uint;
pub type GLsizei = c_int;
pub type GLfloat = c_float;
pub type GLclampf = c_float;
pub type GLchar = c_char;
pub type GLboolean = c_uchar;
pub type GLintptr = c_long;
pub type GLsizeiptr = c_long;

pub type GLFWerrorfun = extern "C" fn(error_code: c_int, description: *const c_char);
pub type GLFWkeyfun =
    extern "C" fn(window: *mut GLFWwindow, key: c_int, scancode: c_int, action: c_int, mods: c_int);

#[allow(clippy::upper_case_acronyms)]
pub type GLDEBUGPROC = extern "C" fn(
    source: GLenum,
    r#type: GLenum,
    id: GLuint,
    severity: GLenum,
    length: GLsizei,
    message: *const GLchar,
    userParam: *const c_void,
);

unsafe extern "C" {
    pub fn glfwGetVersionString() -> *const c_char;

    pub fn glfwInit() -> c_int;
    pub fn glfwTerminate();

    pub fn glfwWindowHint(hint: c_int, value: c_int);

    pub fn glfwCreateWindow(
        width: c_int,
        height: c_int,
        title: *const c_char,
        monitor: *mut GLFWmonitor,
        share: *mut GLFWwindow,
    ) -> *mut GLFWwindow;
    pub fn glfwDestroyWindow(window: *mut GLFWwindow);

    pub fn glfwSetErrorCallback(callback: GLFWerrorfun) -> GLFWerrorfun;

    pub fn glfwSetKeyCallback(window: *mut GLFWwindow, callback: GLFWkeyfun) -> GLFWkeyfun;
    pub fn glfwSetWindowShouldClose(window: *mut GLFWwindow, value: c_int);

    pub fn glfwMakeContextCurrent(window: *mut GLFWwindow);
    pub fn glfwSwapInterval(interval: c_int);

    pub fn glfwWindowShouldClose(window: *mut GLFWwindow) -> c_int;
    pub fn glfwPollEvents();
    pub fn glfwSwapBuffers(window: *mut GLFWwindow);

    pub fn glfwGetKey(window: *mut GLFWwindow, key: c_int) -> c_int;
    pub fn glfwGetCursorPos(window: *mut GLFWwindow, xpos: *mut c_double, ypos: *mut c_double);

    // NOTE: See `https://www.khronos.org/opengl/wiki/OpenGL_Error`.
    pub fn glDebugMessageCallback(callback: GLDEBUGPROC, userParam: *const c_void);

    pub fn glViewport(x: GLint, y: GLint, width: GLsizei, height: GLsizei);

    pub fn glEnable(cap: GLenum);
    pub fn glBlendFunc(sfactor: GLenum, dfactor: GLenum);

    pub fn glClearColor(red: GLclampf, green: GLclampf, blue: GLclampf, alpha: GLclampf);
    pub fn glClear(mask: GLbitfield);

    pub fn glLineWidth(width: GLfloat);

    pub fn glCreateShader(r#type: GLenum) -> GLuint;
    pub fn glShaderSource(
        shader: GLuint,
        count: GLsizei,
        string: *const *const GLchar,
        length: *const GLint,
    );
    pub fn glCompileShader(shader: GLuint);
    pub fn glDeleteShader(shader: GLuint);

    pub fn glCreateProgram() -> GLuint;
    pub fn glDeleteProgram(program: GLuint);
    pub fn glAttachShader(program: GLuint, shader: GLuint);
    pub fn glLinkProgram(program: GLuint);
    pub fn glUseProgram(program: GLuint);

    pub fn glGenVertexArrays(n: GLsizei, arrays: *mut GLuint);
    pub fn glBindVertexArray(array: GLuint);
    pub fn glDeleteVertexArrays(n: GLsizei, arrays: *const GLuint);

    pub fn glGetAttribLocation(program: GLuint, name: *const GLchar) -> GLint;
    pub fn glEnableVertexAttribArray(index: GLuint);
    pub fn glVertexAttribPointer(
        index: GLuint,
        size: GLint,
        r#type: GLenum,
        normalized: GLboolean,
        stride: GLsizei,
        pointer: *const c_void,
    );
    pub fn glVertexAttribDivisor(index: GLuint, divisor: GLuint);

    pub fn glGenBuffers(n: GLsizei, buffers: *mut GLuint);
    pub fn glBindBuffer(target: GLenum, buffer: GLuint);
    pub fn glBufferData(target: GLenum, size: GLsizeiptr, data: *const c_void, usage: GLenum);
    pub fn glBufferSubData(target: GLenum, offset: GLintptr, size: GLsizeiptr, data: *const c_void);
    pub fn glDeleteBuffers(n: GLsizei, buffers: *const GLuint);

    pub fn glGetUniformLocation(program: GLuint, name: *const GLchar) -> GLint;
    pub fn glUniformMatrix4fv(
        location: GLint,
        count: GLsizei,
        transpose: GLboolean,
        value: *const GLfloat,
    );

    // pub fn glDrawArrays(mode: GLenum, first: GLint, count: GLsizei);
    pub fn glDrawArraysInstanced(
        mode: GLenum,
        first: GLint,
        count: GLsizei,
        instancecount: GLsizei,
    );
}

pub const GLFW_RESIZABLE: c_int = 0x0002_0003;
pub const GLFW_SAMPLES: c_int = 0x0002_100D;
pub const GLFW_CONTEXT_VERSION_MAJOR: c_int = 0x0002_2002;
pub const GLFW_CONTEXT_VERSION_MINOR: c_int = 0x0002_2003;
pub const GLFW_OPENGL_DEBUG_CONTEXT: c_int = 0x0002_2007;
pub const GLFW_OPENGL_PROFILE: c_int = 0x0002_2008;
pub const GLFW_OPENGL_CORE_PROFILE: c_int = 0x0003_2001;

pub const GLFW_PRESS: c_int = 1;

pub const GLFW_KEY_ESCAPE: c_int = 256;
pub const GLFW_KEY_W: c_int = 87;
pub const GLFW_KEY_S: c_int = 83;
pub const GLFW_KEY_A: c_int = 65;
pub const GLFW_KEY_D: c_int = 68;

pub const GL_FALSE: GLboolean = 0;

pub const GL_FLOAT: GLenum = 0x1406;

pub const GL_LINES: GLenum = 0x0001;
// pub const GL_TRIANGLES: GLenum = 0x0004;
pub const GL_TRIANGLE_STRIP: GLenum = 0x0005;

pub const GL_LINE_SMOOTH: GLenum = 0x0B20;

pub const GL_VERTEX_SHADER: GLenum = 0x8B31;
pub const GL_FRAGMENT_SHADER: GLenum = 0x8B30;

pub const GL_ARRAY_BUFFER: GLenum = 0x8892;

pub const GL_STATIC_DRAW: GLenum = 0x88E4;
pub const GL_DYNAMIC_DRAW: GLenum = 0x88E8;

// pub const GL_DEBUG_TYPE_ERROR: GLenum = 0x824C;
pub const GL_DEBUG_OUTPUT: GLenum = 0x92E0;
pub const GL_DEBUG_OUTPUT_SYNCHRONOUS: GLenum = 0x8242;

// pub const GL_DEPTH_BUFFER_BIT: GLbitfield = 0x0000_0100;
// pub const GL_STENCIL_BUFFER_BIT: GLbitfield = 0x0000_0400;
pub const GL_COLOR_BUFFER_BIT: GLbitfield = 0x0000_4000;

pub const GL_BLEND: GLenum = 0x0BE2;
pub const GL_MULTISAMPLE: GLenum = 0x809D;

pub const GL_SRC_ALPHA: GLenum = 0x0302;
pub const GL_ONE_MINUS_SRC_ALPHA: GLenum = 0x0303;

// pub const GL_DEBUG_SOURCE_APPLICATION: GLenum = 0x824A;
// pub const GL_DEBUG_TYPE_OTHER: GLenum = 0x8251;
// pub const GL_DEBUG_SEVERITY_NOTIFICATION: GLenum = 0x826B;
