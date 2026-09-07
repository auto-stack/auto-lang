import { Fragment as e, computed as t, createBlock as n, createCommentVNode as r, createElementBlock as i, createElementVNode as a, createStaticVNode as o, createTextVNode as s, createVNode as c, defineComponent as l, h as u, mergeModels as d, nextTick as f, normalizeClass as p, normalizeStyle as m, onBeforeUnmount as h, onMounted as g, onUnmounted as _, openBlock as v, ref as y, renderList as b, renderSlot as x, shallowRef as S, toDisplayString as C, unref as w, useModel as T, vModelSelect as E, vShow as D, watch as O, withCtx as k, withDirectives as A } from "vue";
import { Compartment as j, EditorState as M, RangeSetBuilder as N, StateEffect as P, StateField as F } from "@codemirror/state";
import { Decoration as I, EditorView as L, GutterMarker as R, gutter as z, highlightActiveLine as B, keymap as V, lineNumbers as H } from "@codemirror/view";
import { defaultKeymap as U, history as W, historyKeymap as ee, indentWithTab as te } from "@codemirror/commands";
import { oneDark as G } from "@codemirror/theme-one-dark";
import { StreamLanguage as K } from "@codemirror/language";
//#region \0rolldown/runtime.js
var q = Object.create, J = Object.defineProperty, Y = Object.getOwnPropertyDescriptor, ne = Object.getOwnPropertyNames, re = Object.getPrototypeOf, ie = Object.prototype.hasOwnProperty, X = (e, t) => () => (t || (e((t = { exports: {} }).exports, t), e = null), t.exports), ae = (e, t, n, r) => {
	if (t && typeof t == "object" || typeof t == "function") for (var i = ne(t), a = 0, o = i.length, s; a < o; a++) s = i[a], !ie.call(e, s) && s !== n && J(e, s, {
		get: ((e) => t[e]).bind(null, s),
		enumerable: !(r = Y(t, s)) || r.enumerable
	});
	return e;
}, oe = (e, t, n) => (n = e == null ? {} : q(re(e)), ae(t || !e || !e.__esModule ? J(n, "default", {
	value: e,
	enumerable: !0
}) : n, e)), se = (e) => e.replace(/([a-z0-9])([A-Z])/g, "$1-$2").toLowerCase(), Z = {
	xmlns: "http://www.w3.org/2000/svg",
	width: 24,
	height: 24,
	viewBox: "0 0 24 24",
	fill: "none",
	stroke: "currentColor",
	"stroke-width": 2,
	"stroke-linecap": "round",
	"stroke-linejoin": "round"
}, ce = ({ size: e, strokeWidth: t = 2, absoluteStrokeWidth: n, color: r, iconNode: i, name: a, class: o, ...s }, { slots: c }) => u("svg", {
	...Z,
	width: e || Z.width,
	height: e || Z.height,
	stroke: r || Z.stroke,
	"stroke-width": n ? Number(t) * 24 / Number(e) : t,
	class: ["lucide", `lucide-${se(a ?? "icon")}`],
	...s
}, [...i.map((e) => u(...e)), ...c.default ? [c.default()] : []]), Q = (e, t) => (n, { slots: r }) => u(ce, {
	...n,
	iconNode: t,
	name: e
}, r), le = Q("ArrowDownIcon", [["path", {
	d: "M12 5v14",
	key: "s699le"
}], ["path", {
	d: "m19 12-7 7-7-7",
	key: "1idqje"
}]]), ue = Q("ArrowUpIcon", [["path", {
	d: "m5 12 7-7 7 7",
	key: "hav0vg"
}], ["path", {
	d: "M12 19V5",
	key: "x0mq9r"
}]]), de = Q("BugIcon", [
	["path", {
		d: "m8 2 1.88 1.88",
		key: "fmnt4t"
	}],
	["path", {
		d: "M14.12 3.88 16 2",
		key: "qol33r"
	}],
	["path", {
		d: "M9 7.13v-1a3.003 3.003 0 1 1 6 0v1",
		key: "d7y7pr"
	}],
	["path", {
		d: "M12 20c-3.3 0-6-2.7-6-6v-3a4 4 0 0 1 4-4h4a4 4 0 0 1 4 4v3c0 3.3-2.7 6-6 6",
		key: "xs1cw7"
	}],
	["path", {
		d: "M12 20v-9",
		key: "1qisl0"
	}],
	["path", {
		d: "M6.53 9C4.6 8.8 3 7.1 3 5",
		key: "32zzws"
	}],
	["path", {
		d: "M6 13H2",
		key: "82j7cp"
	}],
	["path", {
		d: "M3 21c0-2.1 1.7-3.9 3.8-4",
		key: "4p0ekp"
	}],
	["path", {
		d: "M20.97 5c0 2.1-1.6 3.8-3.5 4",
		key: "18gb23"
	}],
	["path", {
		d: "M22 13h-4",
		key: "1jl80f"
	}],
	["path", {
		d: "M17.2 17c2.1.1 3.8 1.9 3.8 4",
		key: "k3fwyw"
	}]
]), fe = Q("CheckIcon", [["path", {
	d: "M20 6 9 17l-5-5",
	key: "1gmf2c"
}]]), pe = Q("ChevronDownIcon", [["path", {
	d: "m6 9 6 6 6-6",
	key: "qrunsl"
}]]), me = Q("ChevronRightIcon", [["path", {
	d: "m9 18 6-6-6-6",
	key: "mthhwq"
}]]), he = Q("CircleCheckIcon", [["circle", {
	cx: "12",
	cy: "12",
	r: "10",
	key: "1mglay"
}], ["path", {
	d: "m9 12 2 2 4-4",
	key: "dzmm74"
}]]), ge = Q("CircleDashedIcon", [
	["path", {
		d: "M10.1 2.182a10 10 0 0 1 3.8 0",
		key: "5ilxe3"
	}],
	["path", {
		d: "M13.9 21.818a10 10 0 0 1-3.8 0",
		key: "11zvb9"
	}],
	["path", {
		d: "M17.609 3.721a10 10 0 0 1 2.69 2.7",
		key: "1iw5b2"
	}],
	["path", {
		d: "M2.182 13.9a10 10 0 0 1 0-3.8",
		key: "c0bmvh"
	}],
	["path", {
		d: "M20.279 17.609a10 10 0 0 1-2.7 2.69",
		key: "1ruxm7"
	}],
	["path", {
		d: "M21.818 10.1a10 10 0 0 1 0 3.8",
		key: "qkgqxc"
	}],
	["path", {
		d: "M3.721 6.391a10 10 0 0 1 2.7-2.69",
		key: "1mcia2"
	}],
	["path", {
		d: "M6.391 20.279a10 10 0 0 1-2.69-2.7",
		key: "1fvljs"
	}]
]), _e = Q("CircleXIcon", [
	["circle", {
		cx: "12",
		cy: "12",
		r: "10",
		key: "1mglay"
	}],
	["path", {
		d: "m15 9-6 6",
		key: "1uzhvr"
	}],
	["path", {
		d: "m9 9 6 6",
		key: "z0biqf"
	}]
]), ve = Q("CodeXmlIcon", [
	["path", {
		d: "m18 16 4-4-4-4",
		key: "1inbqp"
	}],
	["path", {
		d: "m6 8-4 4 4 4",
		key: "15zrgr"
	}],
	["path", {
		d: "m14.5 4-5 16",
		key: "e7oirm"
	}]
]), ye = Q("CopyIcon", [["rect", {
	width: "14",
	height: "14",
	x: "8",
	y: "8",
	rx: "2",
	ry: "2",
	key: "17jyea"
}], ["path", {
	d: "M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2",
	key: "zix9uf"
}]]), be = Q("ExternalLinkIcon", [
	["path", {
		d: "M15 3h6v6",
		key: "1q9fwt"
	}],
	["path", {
		d: "M10 14 21 3",
		key: "gplh6r"
	}],
	["path", {
		d: "M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6",
		key: "a6xqqp"
	}]
]), xe = Q("FileCodeIcon", [
	["path", {
		d: "M10 12.5 8 15l2 2.5",
		key: "1tg20x"
	}],
	["path", {
		d: "m14 12.5 2 2.5-2 2.5",
		key: "yinavb"
	}],
	["path", {
		d: "M14 2v4a2 2 0 0 0 2 2h4",
		key: "tnqrlb"
	}],
	["path", {
		d: "M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7z",
		key: "1mlx9k"
	}]
]), Se = Q("LoaderCircleIcon", [["path", {
	d: "M21 12a9 9 0 1 1-6.219-8.56",
	key: "13zald"
}]]), Ce = Q("PlayIcon", [["polygon", {
	points: "6 3 20 12 6 21 6 3",
	key: "1oa8hb"
}]]), we = Q("RefreshCwIcon", [
	["path", {
		d: "M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8",
		key: "v9h5vc"
	}],
	["path", {
		d: "M21 3v5h-5",
		key: "1q7to0"
	}],
	["path", {
		d: "M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16",
		key: "3uifl3"
	}],
	["path", {
		d: "M8 16H3v5",
		key: "1cv678"
	}]
]), Te = Q("SearchIcon", [["circle", {
	cx: "11",
	cy: "11",
	r: "8",
	key: "4ej97u"
}], ["path", {
	d: "m21 21-4.3-4.3",
	key: "1qie3q"
}]]), Ee = Q("Share2Icon", [
	["circle", {
		cx: "18",
		cy: "5",
		r: "3",
		key: "gq8acd"
	}],
	["circle", {
		cx: "6",
		cy: "12",
		r: "3",
		key: "w7nqdw"
	}],
	["circle", {
		cx: "18",
		cy: "19",
		r: "3",
		key: "1xt0gg"
	}],
	["line", {
		x1: "8.59",
		x2: "15.42",
		y1: "13.51",
		y2: "17.49",
		key: "47mynk"
	}],
	["line", {
		x1: "15.41",
		x2: "8.59",
		y1: "6.51",
		y2: "10.49",
		key: "1n3mei"
	}]
]), De = Q("SkipForwardIcon", [["polygon", {
	points: "5 4 15 12 5 20 5 4",
	key: "16p6eg"
}], ["line", {
	x1: "19",
	x2: "19",
	y1: "5",
	y2: "19",
	key: "futhcm"
}]]), Oe = Q("SquareIcon", [["rect", {
	width: "18",
	height: "18",
	x: "3",
	y: "3",
	rx: "2",
	key: "afitv7"
}]]), ke = new Set(/* @__PURE__ */ "fn.let.mut.const.var.type.union.enum.tag.alias.spec.ext.static.shared.impl.node.if.else.for.break.continue.loop.is.in.on.as.to.return.next.view.move.copy.take.hold.true.false.nil.null.None.Some.Ok.Err.task.spawn.await.reply.go.use.pac.super.dep.has.and.or.routes.outlet.link.route.nav.grid".split(".")), Ae = new Set(/* @__PURE__ */ "int.uint.byte.i8.i16.i64.u8.u16.u64.usize.float.double.bool.char.void.str.String.cstr.Handle.linear.List.Map.Set.Option.Result.Link".split("."));
function je(e) {
	return e >= "0" && e <= "9";
}
function Me(e) {
	return je(e) || e >= "a" && e <= "f" || e >= "A" && e <= "F";
}
function Ne(e) {
	return /[\p{L}_]/u.test(e);
}
function Pe(e) {
	return /[\p{L}\p{N}_-]/u.test(e);
}
var Fe = K.define({
	name: "auto",
	startState() {
		return {
			inString: !1,
			stringType: "",
			inComment: !1,
			inFString: !1,
			inChar: !1,
			inRawString: !1,
			inMultilineString: !1
		};
	},
	token(e, t) {
		if (t.inComment) return e.match("*/") ? (t.inComment = !1, "comment") : (e.next(), "comment");
		if (t.inMultilineString) return e.match("\"\"\"") ? (t.inMultilineString = !1, "string") : (e.next(), "string");
		if (e.eatSpace()) return null;
		let n = e.peek();
		if (!n) return null;
		if (n === "/" && e.match("//")) return e.skipToEnd(), "comment";
		if (n === "/" && e.match("/*")) return t.inComment = !0, "comment";
		if (n === "\"" && e.match("\"\"\"")) return t.inMultilineString = !0, "string";
		if (n === "'") return e.next(), e.match("\\"), e.next(), e.peek() === "'" && e.next(), "string";
		if (n === "c" && e.match("c\"")) {
			for (; !e.eol() && e.peek() !== "\"";) e.peek() === "\\" && e.next(), e.next();
			return e.peek() === "\"" && e.next(), "string";
		}
		if (n === "f" && (e.match("f\"") || e.match("f`"))) {
			let t = e.string[e.pos - 1];
			for (; !e.eol() && e.peek() !== t;) if (e.peek() === "\\") e.next(), e.next();
			else if (e.peek() === "$" && e.string[e.pos + 1] === "{") return "string";
			else e.next();
			return e.peek() === t && e.next(), "string";
		}
		if (n === "`") {
			for (e.next(); !e.eol() && e.peek() !== "`";) {
				if (e.peek() === "$" && e.string[e.pos + 1] === "{") return "string";
				e.next();
			}
			return e.peek() === "`" && e.next(), "string";
		}
		if (n === "\"") {
			for (e.next(); !e.eol() && e.peek() !== "\"";) e.peek() === "\\" && e.next(), e.next();
			return e.peek() === "\"" && e.next(), "string";
		}
		return je(n) || n === "." && je(e.string[e.pos + 1] || "") || n === "0" && (e.string[e.pos + 1] === "x" || e.string[e.pos + 1] === "b") ? Ie(e) : Ne(n) ? Le(e) : n === "#" ? (e.next(), e.match("if") || e.match("for") || e.match("is") ? "keyword" : e.match("[") ? "meta" : e.match("{") ? "macroName" : "operator") : n === "@" ? (e.next(), Ne(e.peek() || "") && e.eatWhile(Pe), "attributeName") : e.match("==") || e.match("!=") || e.match("<=") || e.match(">=") || e.match("->") || e.match("=>") || e.match("..=") || e.match("??") || e.match("?.") || e.match(".?") || e.match("&&") || e.match("||") || e.match("+=") || e.match("-=") || e.match("*=") || e.match("/=") || e.match("%=") || n === "." && e.match("..") ? "operator" : n === "." && /[a-zA-Z]/.test(e.string[e.pos + 1] || "") ? (e.next(), e.eatWhile(/[a-zA-Z]/), "propertyName") : "+-*/%=<>!&|~:;,.[](){}".indexOf(n) >= 0 ? (e.next(), "operator") : (e.next(), null);
	},
	languageData: { commentTokens: {
		line: "//",
		block: {
			open: "/*",
			close: "*/"
		}
	} }
});
function Ie(e) {
	let t = e.pos, n = e.peek();
	return n === "0" && (e.string[t + 1] === "x" || e.string[t + 1] === "X") ? (e.next(), e.next(), e.eatWhile(Me), e.eatWhile(/[uUiIfFdD]/), "number") : n === "0" && (e.string[t + 1] === "b" || e.string[t + 1] === "B") ? (e.next(), e.next(), e.eatWhile(/[01]/), "number") : (e.eatWhile(je), e.eatWhile(/[_]/), e.eatWhile(je), e.peek() === "." && je(e.string[e.pos + 1] || "") && (e.next(), e.eatWhile(je), e.eatWhile(/[_]/), e.eatWhile(je)), (e.peek() === "e" || e.peek() === "E") && (e.next(), (e.peek() === "-" || e.peek() === "+") && e.next(), e.eatWhile(je)), e.match("usize") || e.match("i64") || e.match("i16") || e.match("i8") || e.match("u64") || e.match("u16") || e.match("u8") || e.match("u") || e.match("f") || e.match("d"), "number");
}
function Le(e) {
	e.eatWhile(Pe);
	let t = e.current();
	return ke.has(t) ? "keyword" : Ae.has(t) ? "typeName" : e.string.slice(e.pos).trimStart()[0] === "(" ? "function" : "variableName";
}
//#endregion
//#region src/utils/floatingScroll.ts
var Re = 24, ze = 700, Be = "floating-scroll-style";
function Ve() {
	try {
		return new URLSearchParams(window.location.search).has("fsbdebug");
	} catch {
		return !1;
	}
}
var He = 0;
function Ue(e, t) {
	let n = window;
	n.__fsbInstances ||= /* @__PURE__ */ new Map(), e === null ? n.__fsbInstances.delete(t) : n.__fsbInstances.set(e.id, e);
	let r = document.getElementById("fsb-hud");
	r && (r.innerHTML = Array.from(n.__fsbInstances.values()).map((e) => `[${e.id}] ${e.target} · scroll:${e.counters.scrollEvents} poll:${e.counters.polls} upd:${e.counters.updates} top:${e.counters.lastScrollTop}→${e.counters.lastThumbTop}`).join("<br>") || "(no floating-scrollbar instances)");
}
function We() {
	if (!Ve() || document.getElementById("fsb-hud")) return;
	let e = document.createElement("div");
	e.id = "fsb-hud", e.style.cssText = "position:fixed;left:8px;bottom:8px;z-index:99999;background:#111c;color:#7ee787;font:11px/1.5 monospace;padding:8px 10px;border:1px solid #369;border-radius:4px;max-width:70vw;pointer-events:none;", document.body.appendChild(e), window.__fsbInstances && Ue(null);
}
function Ge() {
	if (document.getElementById(Be)) return;
	let e = document.createElement("style");
	e.id = Be, e.textContent = "\n.fsb-rail {\n  position: absolute;\n  opacity: 0;\n  pointer-events: none;\n  transition: opacity 0.2s ease;\n  z-index: 8;\n}\n.fsb-rail.visible {\n  opacity: 1;\n  pointer-events: auto;\n}\n.fsb-y {\n  top: 3px;\n  right: 3px;\n  width: 12px;\n  height: calc(100% - 6px);\n}\n.fsb-x {\n  bottom: 3px;\n  left: 3px;\n  height: 12px;\n  width: calc(100% - 6px);\n}\n.fsb-thumb {\n  /* absolute inside the positioned rail so style.top/left actually move it */\n  position: absolute;\n  background: rgba(212, 212, 212, 0.18);\n  border-radius: 3px;\n  transition: background 0.15s ease;\n}\n.fsb-y .fsb-thumb {\n  left: 2.5px;\n  width: 7px;\n}\n.fsb-x .fsb-thumb {\n  top: 2.5px;\n  height: 7px;\n}\n.fsb-rail:hover .fsb-thumb,\n.fsb-thumb.dragging {\n  background: rgba(212, 212, 212, 0.35);\n}\n", document.head.appendChild(e);
}
function Ke(e, t) {
	Ge(), We();
	let n = ++He, r = Ve() ? {
		id: n,
		target: `${e.tagName.toLowerCase()}.${(e.className || "").split(" ")[0]}`,
		counters: {
			scrollEvents: 0,
			polls: 0,
			updates: 0,
			lastScrollTop: null,
			lastThumbTop: null
		}
	} : null;
	r && Ue(r);
	let i = t ?? e.parentElement, a = i.style.position;
	getComputedStyle(i).position === "static" && (i.style.position = "relative");
	let o = document.createElement("div");
	o.className = "fsb-rail fsb-y";
	let s = document.createElement("div");
	s.className = "fsb-thumb", o.appendChild(s);
	let c = document.createElement("div");
	c.className = "fsb-rail fsb-x";
	let l = document.createElement("div");
	l.className = "fsb-thumb", c.appendChild(l), i.appendChild(o), i.appendChild(c);
	let u = !1, d = null, f = 0, p = !1, m = window.setInterval(() => {
		p || (r ? (r.counters.polls++, (u || T) && x(), Ue(r)) : (u || T) && x());
	}, 120);
	function h() {
		p || (x(), u = !0, _(), d && clearTimeout(d));
	}
	function g() {
		d && clearTimeout(d), d = setTimeout(() => {
			T || (u = !1, _());
		}, ze);
	}
	function _() {
		o.classList.toggle("visible", u && y), c.classList.toggle("visible", u && b);
	}
	function v(e, t, n) {
		return Math.min(n, Math.max(t, e));
	}
	let y = !1, b = !1;
	function x() {
		if (p) return;
		let t = e.scrollHeight - e.clientHeight, n = e.scrollWidth - e.clientWidth;
		y = t > 1, b = n > 1, o.style.display = y ? "" : "none", c.style.display = b ? "" : "none";
		let i = e.clientHeight - 6, a = e.clientWidth - 6;
		if (y) {
			let n = Math.max(Re, Math.round(e.clientHeight / e.scrollHeight * i)), a = Math.round(e.scrollTop / t * (i - n));
			s.style.height = n + "px", s.style.top = a + "px", r && (r.counters.updates++, r.counters.lastScrollTop = Math.round(e.scrollTop), r.counters.lastThumbTop = a);
		}
		if (b) {
			let t = Math.max(Re, Math.round(e.clientWidth / e.scrollWidth * a)), r = Math.round(e.scrollLeft / n * (a - t));
			l.style.width = t + "px", l.style.left = r + "px";
		}
		_();
	}
	function S() {
		cancelAnimationFrame(f), f = requestAnimationFrame(x);
	}
	function C() {
		r && r.counters.scrollEvents++, x(), S(), h(), g();
	}
	function w(t, n) {
		T = n;
		let r = t.clientY, i = t.clientX, a = e.scrollTop, u = e.scrollLeft, d = e.scrollHeight / Math.max(1, e.clientHeight - 6), f = e.scrollWidth / Math.max(1, e.clientWidth - 6), p = n === "y" ? o : c;
		try {
			p.setPointerCapture(t.pointerId);
		} catch {}
		let m = (t) => {
			n === "y" ? e.scrollTop = a + (t.clientY - r) * d : e.scrollLeft = u + (t.clientX - i) * f, x();
		}, h = () => {
			T = null, s.classList.remove("dragging"), l.classList.remove("dragging"), window.removeEventListener("pointermove", m), window.removeEventListener("pointerup", h), window.removeEventListener("pointercancel", h), g();
		};
		s.classList.toggle("dragging", n === "y"), l.classList.toggle("dragging", n === "x"), window.addEventListener("pointermove", m), window.addEventListener("pointerup", h), window.addEventListener("pointercancel", h);
	}
	let T = null;
	function E(t, n) {
		t.preventDefault(), t.stopPropagation(), h();
		let r = e.getBoundingClientRect();
		if (n === "y") {
			let n = e.clientHeight - 6, i = t.clientY - r.top - 3, a = D();
			e.scrollTop = v(i - a / 2, 0, n - a) / Math.max(1, n - a) * (e.scrollHeight - e.clientHeight);
		} else {
			let n = e.clientWidth - 6, i = t.clientX - r.left - 3, a = O();
			e.scrollLeft = v(i - a / 2, 0, n - a) / Math.max(1, n - a) * (e.scrollWidth - e.clientWidth);
		}
		x(), w(t, n);
	}
	function D() {
		return parseFloat(s.style.height || String(Re));
	}
	function O() {
		return parseFloat(l.style.width || String(Re));
	}
	function k(t) {
		t.preventDefault(), t.stopPropagation(), t.deltaY !== 0 && (e.scrollTop += t.deltaY), t.deltaX !== 0 && (e.scrollLeft += t.deltaX), h(), g();
	}
	function A() {
		h(), x();
	}
	function j() {
		g();
	}
	o.addEventListener("pointerdown", (e) => E(e, "y")), c.addEventListener("pointerdown", (e) => E(e, "x")), o.addEventListener("wheel", k, { passive: !1 }), c.addEventListener("wheel", k, { passive: !1 }), e.addEventListener("scroll", C, { passive: !0 }), i.addEventListener("mouseenter", A), i.addEventListener("mouseleave", j);
	let M = new ResizeObserver(() => x());
	M.observe(e);
	let N = new MutationObserver(() => x());
	return N.observe(e, {
		childList: !0,
		subtree: !0,
		characterData: !0
	}), window.addEventListener("resize", x), x(), { destroy() {
		p = !0, window.clearInterval(m), r && Ue(null, n), M.disconnect(), N.disconnect(), window.removeEventListener("resize", x), e.removeEventListener("scroll", C), i.removeEventListener("mouseenter", A), i.removeEventListener("mouseleave", j), o.remove(), c.remove(), a !== void 0 && (i.style.position = a), d && clearTimeout(d), cancelAnimationFrame(f);
	} };
}
//#endregion
//#region src/components/CodeEditor.vue?vue&type=script&setup=true&lang.ts
var qe = /* @__PURE__ */ l({
	__name: "CodeEditor",
	props: {
		modelValue: {},
		onRun: { type: Function },
		isDebugging: { type: Boolean },
		breakpoints: {},
		currentDebugLine: {},
		highlightedSourceLine: {},
		selectedSourceLine: {},
		errorLines: {},
		readOnly: { type: Boolean }
	},
	emits: [
		"update:modelValue",
		"line-click",
		"breakpointsChange",
		"hover-line",
		"hover-line-leave"
	],
	setup(e, { emit: t }) {
		let n = e, r = t, a = y(), o = null, s = null, c = new j(), l = P.define(), u = F.define({
			create() {
				return /* @__PURE__ */ new Set();
			},
			update(e, t) {
				for (let n of t.effects) if (n.is(l)) {
					let t = n.value, r = new Set(e);
					return r.has(t) ? r.delete(t) : r.add(t), r;
				}
				return e;
			}
		});
		class d extends R {
			eq(e) {
				return e instanceof d;
			}
			toDOM() {
				let e = document.createElement("div");
				return e.style.width = "10px", e.style.height = "10px", e.style.borderRadius = "50%", e.style.background = "#e51400", e.className = "cm-breakpoint-marker", e;
			}
		}
		class f extends R {
			eq(e) {
				return e instanceof f;
			}
			toDOM() {
				let e = document.createElement("div");
				return e.style.width = "10px", e.style.height = "10px", e.style.borderRadius = "50%", e.style.border = "1.5px solid #e51400", e.style.background = "transparent", e.style.opacity = "0", e.style.transition = "opacity 0.15s ease", e.className = "cm-empty-circle-marker", e;
			}
		}
		class p extends R {
			eq(e) {
				return e instanceof p;
			}
			toDOM() {
				let e = document.createElement("div");
				return e.style.width = "22px", e.className = "cm-breakpoint-spacer", e;
			}
		}
		let m = [u, z({
			class: "cm-breakpoint-gutter",
			markers(e) {
				let t = new N(), n = e.state.field(u);
				for (let r = 1; r <= e.state.doc.lines; r++) {
					let i = e.state.doc.line(r), a = n.has(r) ? new d() : new f();
					t.add(i.from, i.from, a);
				}
				return t.finish();
			},
			initialSpacer() {
				return new p();
			},
			domEventHandlers: { mousedown(e, t) {
				let n = e.state.doc.lineAt(t.from).number;
				e.dispatch({ effects: l.of(n) });
				let i = e.state.field(u);
				return r("breakpointsChange", Array.from(i)), r("line-click", n), !0;
			} }
		})], h = P.define(), b = P.define(), x = P.define(), S = F.define({
			create() {
				return I.none;
			},
			update(e, t) {
				for (let e of t.effects) if (e.is(h)) {
					if (e.value === null || e.value <= 0) return I.none;
					let n = t.state.doc.line(e.value);
					return I.set([I.line({ class: "cm-debug-current-line" }).range(n.from)]);
				}
				return e.map(t.changes);
			},
			provide: (e) => L.decorations.from(e)
		}), C = F.define({
			create() {
				return I.none;
			},
			update(e, t) {
				for (let e of t.effects) if (e.is(b)) {
					if (e.value === null || e.value <= 0) return I.none;
					let n = t.state.doc.line(e.value);
					return I.set([I.line({ class: "cm-cross-highlight-line" }).range(n.from)]);
				}
				return e.map(t.changes);
			},
			provide: (e) => L.decorations.from(e)
		}), w = F.define({
			create() {
				return I.none;
			},
			update(e, t) {
				for (let e of t.effects) if (e.is(x)) {
					if (e.value === null || e.value <= 0) return I.none;
					let n = t.state.doc.line(e.value);
					return I.set([I.line({ class: "cm-selected-line" }).range(n.from)]);
				}
				return e.map(t.changes);
			},
			provide: (e) => L.decorations.from(e)
		}), T = P.define(), E = [
			C,
			w,
			F.define({
				create() {
					return I.none;
				},
				update(e, t) {
					for (let e of t.effects) if (e.is(T)) {
						if (!e.value || e.value.length === 0) return I.none;
						let n = e.value.filter((e) => e > 0 && e <= t.state.doc.lines).map((e) => {
							let n = t.state.doc.line(e);
							return I.line({ class: "cm-error-line" }).range(n.from);
						});
						return I.set(n);
					}
					return e.map(t.changes);
				},
				provide: (e) => L.decorations.from(e)
			}),
			L.baseTheme({
				".cm-cross-highlight-line": { backgroundColor: "rgba(86, 156, 214, 0.16)" },
				".cm-selected-line": { backgroundColor: "rgba(255, 157, 0, 0.2)" },
				".cm-error-line": {
					backgroundColor: "#f38ba822",
					borderLeft: "3px solid #f38ba8"
				}
			})
		], D = [
			S,
			m,
			L.baseTheme({
				".cm-debug-current-line": {
					backgroundColor: "#0e639c40",
					borderLeft: "3px solid #0e639c"
				},
				".cm-breakpoint-gutter": { width: "22px" },
				".cm-breakpoint-gutter .cm-gutterElement": {
					display: "flex",
					alignItems: "center",
					justifyContent: "center"
				},
				".cm-empty-circle-marker, .cm-breakpoint-marker": { marginTop: "1px" },
				".cm-gutterElement:hover .cm-empty-circle-marker": { opacity: "1 !important" }
			})
		];
		function k() {
			return D;
		}
		return g(() => {
			if (!a.value) return;
			let e = [
				H(),
				B(),
				W(),
				V.of([
					...U,
					...ee,
					te
				]),
				Fe,
				G,
				L.updateListener.of((e) => {
					e.docChanged && !n.readOnly && r("update:modelValue", e.state.doc.toString());
				}),
				...E,
				c.of(n.isDebugging ? k() : [])
			];
			n.readOnly && e.push(L.editable.of(!1)), n.onRun && e.push(V.of([{
				key: "Ctrl-Enter",
				run: () => (n.isDebugging || n.onRun?.(), !0)
			}])), e.push(L.domEventHandlers({ mousedown(e, t) {
				if (e.target.closest(".cm-gutters")) return !1;
				let n = t.posAtCoords({
					x: e.clientX,
					y: e.clientY
				});
				return n == null || r("line-click", t.state.doc.lineAt(n).number), !1;
			} })), o = new L({
				state: M.create({
					doc: n.modelValue,
					extensions: e
				}),
				parent: a.value
			}), s = Ke(o.scrollDOM, a.value);
			let t = 0;
			a.value.addEventListener("mousemove", (e) => {
				if (!o) return;
				let n = o.posAtCoords({
					x: e.clientX,
					y: e.clientY
				});
				if (n !== null) {
					let e = o.state.doc.lineAt(n).number;
					e !== t && (t = e, r("hover-line", e));
				}
			}), a.value.addEventListener("mouseleave", () => {
				t = 0, r("hover-line-leave");
			});
			let i = a.value.querySelector(".cm-lineNumbers");
			i && i.addEventListener("click", (e) => {
				let t = e.target.closest(".cm-gutterElement");
				if (t && t.textContent) {
					let e = parseInt(t.textContent.trim(), 10);
					isNaN(e) || r("line-click", e);
				}
			});
		}), O(() => n.modelValue, (e) => {
			o && o.state.doc.toString() !== e && o.dispatch({ changes: {
				from: 0,
				to: o.state.doc.length,
				insert: e
			} });
		}), O(() => n.isDebugging, (e) => {
			o && o.dispatch({ effects: c.reconfigure(e ? k() : []) });
		}), O(() => n.currentDebugLine, (e) => {
			o && o.dispatch({ effects: h.of(e ?? null) });
		}), O(() => n.highlightedSourceLine, (e) => {
			if (!o) return;
			let t = [b.of(e ?? null)];
			if (e != null) {
				let n = Math.min(o.state.doc.length, o.state.doc.line(e).from);
				t.push(L.scrollIntoView(n, { y: "nearest" }));
			}
			o.dispatch({ effects: t });
		}), O(() => n.selectedSourceLine, (e) => {
			o && o.dispatch({ effects: [x.of(e ?? null)] });
		}), O(() => n.errorLines, (e) => {
			o && o.dispatch({ effects: T.of(e ?? []) });
		}), O(() => n.breakpoints, (e) => {
			if (!o) return;
			let t = o.state.field(u, !1);
			if (!t) return;
			let n = Array.from(t), r = e || [], i = r.filter((e) => !n.includes(e)), a = n.filter((e) => !r.includes(e));
			if (i.length === 0 && a.length === 0) return;
			let s = [...i.map((e) => l.of(e)), ...a.map((e) => l.of(e))];
			o.dispatch({ effects: s });
		}, { deep: !0 }), _(() => {
			s?.destroy(), s = null, o?.destroy();
		}), (e, t) => (v(), i("div", {
			ref_key: "editorContainer",
			ref: a,
			class: "editor-container"
		}, null, 512));
	}
}), $ = (e, t) => {
	let n = e.__vccOpts || e;
	for (let [e, r] of t) n[e] = r;
	return n;
}, Je = /* @__PURE__ */ $(qe, [["__scopeId", "data-v-b2c9dc90"]]), Ye = /* @__PURE__ */ $(/* @__PURE__ */ l({
	__name: "ScrollArea",
	setup(e, { expose: t }) {
		let n = y(null), r = y(null), o = null;
		return g(() => {
			!n.value || !r.value || (o = Ke(r.value, n.value));
		}), h(() => {
			o?.destroy(), o = null;
		}), t({
			scrollTo(e) {
				r.value?.scrollTo(e);
			},
			get scrollTop() {
				return r.value?.scrollTop ?? 0;
			},
			set scrollTop(e) {
				r.value && (r.value.scrollTop = e);
			}
		}), (e, t) => (v(), i("div", {
			ref_key: "rootEl",
			ref: n,
			class: "scroll-area"
		}, [a("div", {
			ref_key: "contentEl",
			ref: r,
			class: "scroll-content"
		}, [x(e.$slots, "default", {}, void 0, !0)], 512)], 512));
	}
}), [["__scopeId", "data-v-be12123c"]]), Xe = (/* @__PURE__ */ oe((/* @__PURE__ */ X(((e, t) => {
	function n(e) {
		return e instanceof Map ? e.clear = e.delete = e.set = function() {
			throw Error("map is read-only");
		} : e instanceof Set && (e.add = e.clear = e.delete = function() {
			throw Error("set is read-only");
		}), Object.freeze(e), Object.getOwnPropertyNames(e).forEach((t) => {
			let r = e[t], i = typeof r;
			(i === "object" || i === "function") && !Object.isFrozen(r) && n(r);
		}), e;
	}
	var r = class {
		constructor(e) {
			e.data === void 0 && (e.data = {}), this.data = e.data, this.isMatchIgnored = !1;
		}
		ignoreMatch() {
			this.isMatchIgnored = !0;
		}
	};
	function i(e) {
		return e.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;").replace(/'/g, "&#x27;");
	}
	function a(e, ...t) {
		let n = Object.create(null);
		for (let t in e) n[t] = e[t];
		return t.forEach(function(e) {
			for (let t in e) n[t] = e[t];
		}), n;
	}
	var o = "</span>", s = (e) => !!e.scope, c = (e, { prefix: t }) => {
		if (e.startsWith("language:")) return e.replace("language:", "language-");
		if (e.includes(".")) {
			let n = e.split(".");
			return [`${t}${n.shift()}`, ...n.map((e, t) => `${e}${"_".repeat(t + 1)}`)].join(" ");
		}
		return `${t}${e}`;
	}, l = class {
		constructor(e, t) {
			this.buffer = "", this.classPrefix = t.classPrefix, e.walk(this);
		}
		addText(e) {
			this.buffer += i(e);
		}
		openNode(e) {
			if (!s(e)) return;
			let t = c(e.scope, { prefix: this.classPrefix });
			this.span(t);
		}
		closeNode(e) {
			s(e) && (this.buffer += o);
		}
		value() {
			return this.buffer;
		}
		span(e) {
			this.buffer += `<span class="${e}">`;
		}
	}, u = (e = {}) => {
		let t = { children: [] };
		return Object.assign(t, e), t;
	}, d = class e {
		constructor() {
			this.rootNode = u(), this.stack = [this.rootNode];
		}
		get top() {
			return this.stack[this.stack.length - 1];
		}
		get root() {
			return this.rootNode;
		}
		add(e) {
			this.top.children.push(e);
		}
		openNode(e) {
			let t = u({ scope: e });
			this.add(t), this.stack.push(t);
		}
		closeNode() {
			if (this.stack.length > 1) return this.stack.pop();
		}
		closeAllNodes() {
			for (; this.closeNode(););
		}
		toJSON() {
			return JSON.stringify(this.rootNode, null, 4);
		}
		walk(e) {
			return this.constructor._walk(e, this.rootNode);
		}
		static _walk(e, t) {
			return typeof t == "string" ? e.addText(t) : t.children && (e.openNode(t), t.children.forEach((t) => this._walk(e, t)), e.closeNode(t)), e;
		}
		static _collapse(t) {
			typeof t != "string" && t.children && (t.children.every((e) => typeof e == "string") ? t.children = [t.children.join("")] : t.children.forEach((t) => {
				e._collapse(t);
			}));
		}
	}, f = class extends d {
		constructor(e) {
			super(), this.options = e;
		}
		addText(e) {
			e !== "" && this.add(e);
		}
		startScope(e) {
			this.openNode(e);
		}
		endScope() {
			this.closeNode();
		}
		__addSublanguage(e, t) {
			let n = e.root;
			t && (n.scope = `language:${t}`), this.add(n);
		}
		toHTML() {
			return new l(this, this.options).value();
		}
		finalize() {
			return this.closeAllNodes(), !0;
		}
	};
	function p(e) {
		return e ? typeof e == "string" ? e : e.source : null;
	}
	function m(e) {
		return _("(?=", e, ")");
	}
	function h(e) {
		return _("(?:", e, ")*");
	}
	function g(e) {
		return _("(?:", e, ")?");
	}
	function _(...e) {
		return e.map((e) => p(e)).join("");
	}
	function v(e) {
		let t = e[e.length - 1];
		return typeof t == "object" && t.constructor === Object ? (e.splice(e.length - 1, 1), t) : {};
	}
	function y(...e) {
		return "(" + (v(e).capture ? "" : "?:") + e.map((e) => p(e)).join("|") + ")";
	}
	function b(e) {
		return RegExp(e.toString() + "|").exec("").length - 1;
	}
	function x(e, t) {
		let n = e && e.exec(t);
		return n && n.index === 0;
	}
	var S = /\[(?:[^\\\]]|\\.)*\]|\(\??|\\([1-9][0-9]*)|\\./;
	function C(e, { joinWith: t }) {
		let n = 0;
		return e.map((e) => {
			n += 1;
			let t = n, r = p(e), i = "";
			for (; r.length > 0;) {
				let e = S.exec(r);
				if (!e) {
					i += r;
					break;
				}
				i += r.substring(0, e.index), r = r.substring(e.index + e[0].length), e[0][0] === "\\" && e[1] ? i += "\\" + String(Number(e[1]) + t) : (i += e[0], e[0] === "(" && n++);
			}
			return i;
		}).map((e) => `(${e})`).join(t);
	}
	var w = /\b\B/, T = "[a-zA-Z]\\w*", E = "[a-zA-Z_]\\w*", D = "\\b\\d+(\\.\\d+)?", O = "(-?)(\\b0[xX][a-fA-F0-9]+|(\\b\\d+(\\.\\d*)?|\\.\\d+)([eE][-+]?\\d+)?)", k = "\\b(0b[01]+)", A = "!|!=|!==|%|%=|&|&&|&=|\\*|\\*=|\\+|\\+=|,|-|-=|/=|/|:|;|<<|<<=|<=|<|===|==|=|>>>=|>>=|>=|>>>|>>|>|\\?|\\[|\\{|\\(|\\^|\\^=|\\||\\|=|\\|\\||~", j = (e = {}) => {
		let t = /^#![ ]*\//;
		return e.binary && (e.begin = _(t, /.*\b/, e.binary, /\b.*/)), a({
			scope: "meta",
			begin: t,
			end: /$/,
			relevance: 0,
			"on:begin": (e, t) => {
				e.index !== 0 && t.ignoreMatch();
			}
		}, e);
	}, M = {
		begin: "\\\\[\\s\\S]",
		relevance: 0
	}, N = {
		scope: "string",
		begin: "'",
		end: "'",
		illegal: "\\n",
		contains: [M]
	}, P = {
		scope: "string",
		begin: "\"",
		end: "\"",
		illegal: "\\n",
		contains: [M]
	}, F = { begin: /\b(a|an|the|are|I'm|isn't|don't|doesn't|won't|but|just|should|pretty|simply|enough|gonna|going|wtf|so|such|will|you|your|they|like|more)\b/ }, I = function(e, t, n = {}) {
		let r = a({
			scope: "comment",
			begin: e,
			end: t,
			contains: []
		}, n);
		r.contains.push({
			scope: "doctag",
			begin: "[ ]*(?=(TODO|FIXME|NOTE|BUG|OPTIMIZE|HACK|XXX):)",
			end: /(TODO|FIXME|NOTE|BUG|OPTIMIZE|HACK|XXX):/,
			excludeBegin: !0,
			relevance: 0
		});
		let i = y("I", "a", "is", "so", "us", "to", "at", "if", "in", "it", "on", /[A-Za-z]+['](d|ve|re|ll|t|s|n)/, /[A-Za-z]+[-][a-z]+/, /[A-Za-z][a-z]{2,}/);
		return r.contains.push({ begin: _(/[ ]+/, "(", i, /[.]?[:]?([.][ ]|[ ])/, "){3}") }), r;
	}, L = I("//", "$"), R = I("/\\*", "\\*/"), z = I("#", "$"), B = {
		scope: "number",
		begin: D,
		relevance: 0
	}, V = {
		scope: "number",
		begin: O,
		relevance: 0
	}, H = {
		scope: "number",
		begin: k,
		relevance: 0
	}, U = {
		scope: "regexp",
		begin: /\/(?=[^/\n]*\/)/,
		end: /\/[gimuy]*/,
		contains: [M, {
			begin: /\[/,
			end: /\]/,
			relevance: 0,
			contains: [M]
		}]
	}, W = {
		scope: "title",
		begin: T,
		relevance: 0
	}, ee = {
		scope: "title",
		begin: E,
		relevance: 0
	}, te = {
		begin: "\\.\\s*" + E,
		relevance: 0
	}, G = /* @__PURE__ */ Object.freeze({
		__proto__: null,
		APOS_STRING_MODE: N,
		BACKSLASH_ESCAPE: M,
		BINARY_NUMBER_MODE: H,
		BINARY_NUMBER_RE: k,
		COMMENT: I,
		C_BLOCK_COMMENT_MODE: R,
		C_LINE_COMMENT_MODE: L,
		C_NUMBER_MODE: V,
		C_NUMBER_RE: O,
		END_SAME_AS_BEGIN: function(e) {
			return Object.assign(e, {
				"on:begin": (e, t) => {
					t.data._beginMatch = e[1];
				},
				"on:end": (e, t) => {
					t.data._beginMatch !== e[1] && t.ignoreMatch();
				}
			});
		},
		HASH_COMMENT_MODE: z,
		IDENT_RE: T,
		MATCH_NOTHING_RE: w,
		METHOD_GUARD: te,
		NUMBER_MODE: B,
		NUMBER_RE: D,
		PHRASAL_WORDS_MODE: F,
		QUOTE_STRING_MODE: P,
		REGEXP_MODE: U,
		RE_STARTERS_RE: A,
		SHEBANG: j,
		TITLE_MODE: W,
		UNDERSCORE_IDENT_RE: E,
		UNDERSCORE_TITLE_MODE: ee
	});
	function K(e, t) {
		e.input[e.index - 1] === "." && t.ignoreMatch();
	}
	function q(e, t) {
		e.className !== void 0 && (e.scope = e.className, delete e.className);
	}
	function J(e, t) {
		t && e.beginKeywords && (e.begin = "\\b(" + e.beginKeywords.split(" ").join("|") + ")(?!\\.)(?=\\b|\\s)", e.__beforeBegin = K, e.keywords = e.keywords || e.beginKeywords, delete e.beginKeywords, e.relevance === void 0 && (e.relevance = 0));
	}
	function Y(e, t) {
		Array.isArray(e.illegal) && (e.illegal = y(...e.illegal));
	}
	function ne(e, t) {
		if (e.match) {
			if (e.begin || e.end) throw Error("begin & end are not supported with match");
			e.begin = e.match, delete e.match;
		}
	}
	function re(e, t) {
		e.relevance === void 0 && (e.relevance = 1);
	}
	var ie = (e, t) => {
		if (!e.beforeMatch) return;
		if (e.starts) throw Error("beforeMatch cannot be used with starts");
		let n = Object.assign({}, e);
		Object.keys(e).forEach((t) => {
			delete e[t];
		}), e.keywords = n.keywords, e.begin = _(n.beforeMatch, m(n.begin)), e.starts = {
			relevance: 0,
			contains: [Object.assign(n, { endsParent: !0 })]
		}, e.relevance = 0, delete n.beforeMatch;
	}, X = [
		"of",
		"and",
		"for",
		"in",
		"not",
		"or",
		"if",
		"then",
		"parent",
		"list",
		"value"
	], ae = "keyword";
	function oe(e, t, n = ae) {
		let r = Object.create(null);
		return typeof e == "string" ? i(n, e.split(" ")) : Array.isArray(e) ? i(n, e) : Object.keys(e).forEach(function(n) {
			Object.assign(r, oe(e[n], t, n));
		}), r;
		function i(e, n) {
			t && (n = n.map((e) => e.toLowerCase())), n.forEach(function(t) {
				let n = t.split("|");
				r[n[0]] = [e, se(n[0], n[1])];
			});
		}
	}
	function se(e, t) {
		return t ? Number(t) : +!Z(e);
	}
	function Z(e) {
		return X.includes(e.toLowerCase());
	}
	var ce = {}, Q = (e) => {
		console.error(e);
	}, le = (e, ...t) => {
		console.log(`WARN: ${e}`, ...t);
	}, ue = (e, t) => {
		ce[`${e}/${t}`] || (console.log(`Deprecated as of ${e}. ${t}`), ce[`${e}/${t}`] = !0);
	}, de = /* @__PURE__ */ Error();
	function fe(e, t, { key: n }) {
		let r = 0, i = e[n], a = {}, o = {};
		for (let e = 1; e <= t.length; e++) o[e + r] = i[e], a[e + r] = !0, r += b(t[e - 1]);
		e[n] = o, e[n]._emit = a, e[n]._multi = !0;
	}
	function pe(e) {
		if (Array.isArray(e.begin)) {
			if (e.skip || e.excludeBegin || e.returnBegin) throw Q("skip, excludeBegin, returnBegin not compatible with beginScope: {}"), de;
			if (typeof e.beginScope != "object" || e.beginScope === null) throw Q("beginScope must be object"), de;
			fe(e, e.begin, { key: "beginScope" }), e.begin = C(e.begin, { joinWith: "" });
		}
	}
	function me(e) {
		if (Array.isArray(e.end)) {
			if (e.skip || e.excludeEnd || e.returnEnd) throw Q("skip, excludeEnd, returnEnd not compatible with endScope: {}"), de;
			if (typeof e.endScope != "object" || e.endScope === null) throw Q("endScope must be object"), de;
			fe(e, e.end, { key: "endScope" }), e.end = C(e.end, { joinWith: "" });
		}
	}
	function he(e) {
		e.scope && typeof e.scope == "object" && e.scope !== null && (e.beginScope = e.scope, delete e.scope);
	}
	function ge(e) {
		he(e), typeof e.beginScope == "string" && (e.beginScope = { _wrap: e.beginScope }), typeof e.endScope == "string" && (e.endScope = { _wrap: e.endScope }), pe(e), me(e);
	}
	function _e(e) {
		function t(t, n) {
			return new RegExp(p(t), "m" + (e.case_insensitive ? "i" : "") + (e.unicodeRegex ? "u" : "") + (n ? "g" : ""));
		}
		class n {
			constructor() {
				this.matchIndexes = {}, this.regexes = [], this.matchAt = 1, this.position = 0;
			}
			addRule(e, t) {
				t.position = this.position++, this.matchIndexes[this.matchAt] = t, this.regexes.push([t, e]), this.matchAt += b(e) + 1;
			}
			compile() {
				this.regexes.length === 0 && (this.exec = () => null);
				let e = this.regexes.map((e) => e[1]);
				this.matcherRe = t(C(e, { joinWith: "|" }), !0), this.lastIndex = 0;
			}
			exec(e) {
				this.matcherRe.lastIndex = this.lastIndex;
				let t = this.matcherRe.exec(e);
				if (!t) return null;
				let n = t.findIndex((e, t) => t > 0 && e !== void 0), r = this.matchIndexes[n];
				return t.splice(0, n), Object.assign(t, r);
			}
		}
		class r {
			constructor() {
				this.rules = [], this.multiRegexes = [], this.count = 0, this.lastIndex = 0, this.regexIndex = 0;
			}
			getMatcher(e) {
				if (this.multiRegexes[e]) return this.multiRegexes[e];
				let t = new n();
				return this.rules.slice(e).forEach(([e, n]) => t.addRule(e, n)), t.compile(), this.multiRegexes[e] = t, t;
			}
			resumingScanAtSamePosition() {
				return this.regexIndex !== 0;
			}
			considerAll() {
				this.regexIndex = 0;
			}
			addRule(e, t) {
				this.rules.push([e, t]), t.type === "begin" && this.count++;
			}
			exec(e) {
				let t = this.getMatcher(this.regexIndex);
				t.lastIndex = this.lastIndex;
				let n = t.exec(e);
				if (this.resumingScanAtSamePosition() && !(n && n.index === this.lastIndex)) {
					let t = this.getMatcher(0);
					t.lastIndex = this.lastIndex + 1, n = t.exec(e);
				}
				return n && (this.regexIndex += n.position + 1, this.regexIndex === this.count && this.considerAll()), n;
			}
		}
		function i(e) {
			let t = new r();
			return e.contains.forEach((e) => t.addRule(e.begin, {
				rule: e,
				type: "begin"
			})), e.terminatorEnd && t.addRule(e.terminatorEnd, { type: "end" }), e.illegal && t.addRule(e.illegal, { type: "illegal" }), t;
		}
		function o(n, r) {
			let a = n;
			if (n.isCompiled) return a;
			[
				q,
				ne,
				ge,
				ie
			].forEach((e) => e(n, r)), e.compilerExtensions.forEach((e) => e(n, r)), n.__beforeBegin = null, [
				J,
				Y,
				re
			].forEach((e) => e(n, r)), n.isCompiled = !0;
			let s = null;
			return typeof n.keywords == "object" && n.keywords.$pattern && (n.keywords = Object.assign({}, n.keywords), s = n.keywords.$pattern, delete n.keywords.$pattern), s ||= /\w+/, n.keywords &&= oe(n.keywords, e.case_insensitive), a.keywordPatternRe = t(s, !0), r && (n.begin ||= /\B|\b/, a.beginRe = t(a.begin), !n.end && !n.endsWithParent && (n.end = /\B|\b/), n.end && (a.endRe = t(a.end)), a.terminatorEnd = p(a.end) || "", n.endsWithParent && r.terminatorEnd && (a.terminatorEnd += (n.end ? "|" : "") + r.terminatorEnd)), n.illegal && (a.illegalRe = t(n.illegal)), n.contains ||= [], n.contains = [].concat(...n.contains.map(function(e) {
				return ye(e === "self" ? n : e);
			})), n.contains.forEach(function(e) {
				o(e, a);
			}), n.starts && o(n.starts, r), a.matcher = i(a), a;
		}
		if (e.compilerExtensions ||= [], e.contains && e.contains.includes("self")) throw Error("ERR: contains `self` is not supported at the top-level of a language.  See documentation.");
		return e.classNameAliases = a(e.classNameAliases || {}), o(e);
	}
	function ve(e) {
		return e ? e.endsWithParent || ve(e.starts) : !1;
	}
	function ye(e) {
		return e.variants && !e.cachedVariants && (e.cachedVariants = e.variants.map(function(t) {
			return a(e, { variants: null }, t);
		})), e.cachedVariants ? e.cachedVariants : ve(e) ? a(e, { starts: e.starts ? a(e.starts) : null }) : Object.isFrozen(e) ? a(e) : e;
	}
	var be = "11.11.1", xe = class extends Error {
		constructor(e, t) {
			super(e), this.name = "HTMLInjectionError", this.html = t;
		}
	}, Se = i, Ce = a, we = Symbol("nomatch"), Te = 7, Ee = function(e) {
		let t = Object.create(null), i = Object.create(null), a = [], o = !0, s = "Could not find the language '{}', did you forget to load/include a language module?", c = {
			disableAutodetect: !0,
			name: "Plain text",
			contains: []
		}, l = {
			ignoreUnescapedHTML: !1,
			throwUnescapedHTML: !1,
			noHighlightRe: /^(no-?highlight)$/i,
			languageDetectRe: /\blang(?:uage)?-([\w-]+)\b/i,
			classPrefix: "hljs-",
			cssSelector: "pre code",
			languages: null,
			__emitter: f
		};
		function u(e) {
			return l.noHighlightRe.test(e);
		}
		function d(e) {
			let t = e.className + " ";
			t += e.parentNode ? e.parentNode.className : "";
			let n = l.languageDetectRe.exec(t);
			if (n) {
				let t = N(n[1]);
				return t || (le(s.replace("{}", n[1])), le("Falling back to no-highlight mode for this block.", e)), t ? n[1] : "no-highlight";
			}
			return t.split(/\s+/).find((e) => u(e) || N(e));
		}
		function p(e, t, n) {
			let r = "", i = "";
			typeof t == "object" ? (r = e, n = t.ignoreIllegals, i = t.language) : (ue("10.7.0", "highlight(lang, code, ...args) has been deprecated."), ue("10.7.0", "Please use highlight(code, options) instead.\nhttps://github.com/highlightjs/highlight.js/issues/2277"), i = e, r = t), n === void 0 && (n = !0);
			let a = {
				code: r,
				language: i
			};
			z("before:highlight", a);
			let o = a.result ? a.result : v(a.language, a.code, n);
			return o.code = a.code, z("after:highlight", o), o;
		}
		function v(e, n, i, a) {
			let c = Object.create(null);
			function u(e, t) {
				return e.keywords[t];
			}
			function d() {
				if (!A.keywords) {
					M.addText(P);
					return;
				}
				let e = 0;
				A.keywordPatternRe.lastIndex = 0;
				let t = A.keywordPatternRe.exec(P), n = "";
				for (; t;) {
					n += P.substring(e, t.index);
					let r = D.case_insensitive ? t[0].toLowerCase() : t[0], i = u(A, r);
					if (i) {
						let [e, a] = i;
						if (M.addText(n), n = "", c[r] = (c[r] || 0) + 1, c[r] <= Te && (F += a), e.startsWith("_")) n += t[0];
						else {
							let n = D.classNameAliases[e] || e;
							m(t[0], n);
						}
					} else n += t[0];
					e = A.keywordPatternRe.lastIndex, t = A.keywordPatternRe.exec(P);
				}
				n += P.substring(e), M.addText(n);
			}
			function f() {
				if (P === "") return;
				let e = null;
				if (typeof A.subLanguage == "string") {
					if (!t[A.subLanguage]) {
						M.addText(P);
						return;
					}
					e = v(A.subLanguage, P, !0, j[A.subLanguage]), j[A.subLanguage] = e._top;
				} else e = S(P, A.subLanguage.length ? A.subLanguage : null);
				A.relevance > 0 && (F += e.relevance), M.__addSublanguage(e._emitter, e.language);
			}
			function p() {
				A.subLanguage == null ? d() : f(), P = "";
			}
			function m(e, t) {
				e !== "" && (M.startScope(t), M.addText(e), M.endScope());
			}
			function h(e, t) {
				let n = 1, r = t.length - 1;
				for (; n <= r;) {
					if (!e._emit[n]) {
						n++;
						continue;
					}
					let r = D.classNameAliases[e[n]] || e[n], i = t[n];
					r ? m(i, r) : (P = i, d(), P = ""), n++;
				}
			}
			function g(e, t) {
				return e.scope && typeof e.scope == "string" && M.openNode(D.classNameAliases[e.scope] || e.scope), e.beginScope && (e.beginScope._wrap ? (m(P, D.classNameAliases[e.beginScope._wrap] || e.beginScope._wrap), P = "") : e.beginScope._multi && (h(e.beginScope, t), P = "")), A = Object.create(e, { parent: { value: A } }), A;
			}
			function _(e, t, n) {
				let i = x(e.endRe, n);
				if (i) {
					if (e["on:end"]) {
						let n = new r(e);
						e["on:end"](t, n), n.isMatchIgnored && (i = !1);
					}
					if (i) {
						for (; e.endsParent && e.parent;) e = e.parent;
						return e;
					}
				}
				if (e.endsWithParent) return _(e.parent, t, n);
			}
			function y(e) {
				return A.matcher.regexIndex === 0 ? (P += e[0], 1) : (R = !0, 0);
			}
			function b(e) {
				let t = e[0], n = e.rule, i = new r(n), a = [n.__beforeBegin, n["on:begin"]];
				for (let n of a) if (n && (n(e, i), i.isMatchIgnored)) return y(t);
				return n.skip ? P += t : (n.excludeBegin && (P += t), p(), !n.returnBegin && !n.excludeBegin && (P = t)), g(n, e), n.returnBegin ? 0 : t.length;
			}
			function C(e) {
				let t = e[0], r = n.substring(e.index), i = _(A, e, r);
				if (!i) return we;
				let a = A;
				A.endScope && A.endScope._wrap ? (p(), m(t, A.endScope._wrap)) : A.endScope && A.endScope._multi ? (p(), h(A.endScope, e)) : a.skip ? P += t : (a.returnEnd || a.excludeEnd || (P += t), p(), a.excludeEnd && (P = t));
				do
					A.scope && M.closeNode(), !A.skip && !A.subLanguage && (F += A.relevance), A = A.parent;
				while (A !== i.parent);
				return i.starts && g(i.starts, e), a.returnEnd ? 0 : t.length;
			}
			function w() {
				let e = [];
				for (let t = A; t !== D; t = t.parent) t.scope && e.unshift(t.scope);
				e.forEach((e) => M.openNode(e));
			}
			let T = {};
			function E(t, r) {
				let a = r && r[0];
				if (P += t, a == null) return p(), 0;
				if (T.type === "begin" && r.type === "end" && T.index === r.index && a === "") {
					if (P += n.slice(r.index, r.index + 1), !o) {
						let t = /* @__PURE__ */ Error(`0 width match regex (${e})`);
						throw t.languageName = e, t.badRule = T.rule, t;
					}
					return 1;
				}
				if (T = r, r.type === "begin") return b(r);
				if (r.type === "illegal" && !i) {
					let e = /* @__PURE__ */ Error("Illegal lexeme \"" + a + "\" for mode \"" + (A.scope || "<unnamed>") + "\"");
					throw e.mode = A, e;
				} else if (r.type === "end") {
					let e = C(r);
					if (e !== we) return e;
				}
				if (r.type === "illegal" && a === "") return P += "\n", 1;
				if (L > 1e5 && L > r.index * 3) throw /* @__PURE__ */ Error("potential infinite loop, way more iterations than matches");
				return P += a, a.length;
			}
			let D = N(e);
			if (!D) throw Q(s.replace("{}", e)), Error("Unknown language: \"" + e + "\"");
			let O = _e(D), k = "", A = a || O, j = {}, M = new l.__emitter(l);
			w();
			let P = "", F = 0, I = 0, L = 0, R = !1;
			try {
				if (D.__emitTokens) D.__emitTokens(n, M);
				else {
					for (A.matcher.considerAll();;) {
						L++, R ? R = !1 : A.matcher.considerAll(), A.matcher.lastIndex = I;
						let e = A.matcher.exec(n);
						if (!e) break;
						let t = E(n.substring(I, e.index), e);
						I = e.index + t;
					}
					E(n.substring(I));
				}
				return M.finalize(), k = M.toHTML(), {
					language: e,
					value: k,
					relevance: F,
					illegal: !1,
					_emitter: M,
					_top: A
				};
			} catch (t) {
				if (t.message && t.message.includes("Illegal")) return {
					language: e,
					value: Se(n),
					illegal: !0,
					relevance: 0,
					_illegalBy: {
						message: t.message,
						index: I,
						context: n.slice(I - 100, I + 100),
						mode: t.mode,
						resultSoFar: k
					},
					_emitter: M
				};
				if (o) return {
					language: e,
					value: Se(n),
					illegal: !1,
					relevance: 0,
					errorRaised: t,
					_emitter: M,
					_top: A
				};
				throw t;
			}
		}
		function b(e) {
			let t = {
				value: Se(e),
				illegal: !1,
				relevance: 0,
				_top: c,
				_emitter: new l.__emitter(l)
			};
			return t._emitter.addText(e), t;
		}
		function S(e, n) {
			n = n || l.languages || Object.keys(t);
			let r = b(e), i = n.filter(N).filter(F).map((t) => v(t, e, !1));
			i.unshift(r);
			let [a, o] = i.sort((e, t) => {
				if (e.relevance !== t.relevance) return t.relevance - e.relevance;
				if (e.language && t.language) {
					if (N(e.language).supersetOf === t.language) return 1;
					if (N(t.language).supersetOf === e.language) return -1;
				}
				return 0;
			}), s = a;
			return s.secondBest = o, s;
		}
		function C(e, t, n) {
			let r = t && i[t] || n;
			e.classList.add("hljs"), e.classList.add(`language-${r}`);
		}
		function w(e) {
			let t = null, n = d(e);
			if (u(n)) return;
			if (z("before:highlightElement", {
				el: e,
				language: n
			}), e.dataset.highlighted) {
				console.log("Element previously highlighted. To highlight again, first unset `dataset.highlighted`.", e);
				return;
			}
			if (e.children.length > 0 && (l.ignoreUnescapedHTML || (console.warn("One of your code blocks includes unescaped HTML. This is a potentially serious security risk."), console.warn("https://github.com/highlightjs/highlight.js/wiki/security"), console.warn("The element with unescaped HTML:"), console.warn(e)), l.throwUnescapedHTML)) throw new xe("One of your code blocks includes unescaped HTML.", e.innerHTML);
			t = e;
			let r = t.textContent, i = n ? p(r, {
				language: n,
				ignoreIllegals: !0
			}) : S(r);
			e.innerHTML = i.value, e.dataset.highlighted = "yes", C(e, n, i.language), e.result = {
				language: i.language,
				re: i.relevance,
				relevance: i.relevance
			}, i.secondBest && (e.secondBest = {
				language: i.secondBest.language,
				relevance: i.secondBest.relevance
			}), z("after:highlightElement", {
				el: e,
				result: i,
				text: r
			});
		}
		function T(e) {
			l = Ce(l, e);
		}
		let E = () => {
			k(), ue("10.6.0", "initHighlighting() deprecated.  Use highlightAll() now.");
		};
		function D() {
			k(), ue("10.6.0", "initHighlightingOnLoad() deprecated.  Use highlightAll() now.");
		}
		let O = !1;
		function k() {
			function e() {
				k();
			}
			if (document.readyState === "loading") {
				O || window.addEventListener("DOMContentLoaded", e, !1), O = !0;
				return;
			}
			document.querySelectorAll(l.cssSelector).forEach(w);
		}
		function A(n, r) {
			let i = null;
			try {
				i = r(e);
			} catch (e) {
				if (Q("Language definition for '{}' could not be registered.".replace("{}", n)), o) Q(e);
				else throw e;
				i = c;
			}
			i.name ||= n, t[n] = i, i.rawDefinition = r.bind(null, e), i.aliases && P(i.aliases, { languageName: n });
		}
		function j(e) {
			delete t[e];
			for (let t of Object.keys(i)) i[t] === e && delete i[t];
		}
		function M() {
			return Object.keys(t);
		}
		function N(e) {
			return e = (e || "").toLowerCase(), t[e] || t[i[e]];
		}
		function P(e, { languageName: t }) {
			typeof e == "string" && (e = [e]), e.forEach((e) => {
				i[e.toLowerCase()] = t;
			});
		}
		function F(e) {
			let t = N(e);
			return t && !t.disableAutodetect;
		}
		function I(e) {
			e["before:highlightBlock"] && !e["before:highlightElement"] && (e["before:highlightElement"] = (t) => {
				e["before:highlightBlock"](Object.assign({ block: t.el }, t));
			}), e["after:highlightBlock"] && !e["after:highlightElement"] && (e["after:highlightElement"] = (t) => {
				e["after:highlightBlock"](Object.assign({ block: t.el }, t));
			});
		}
		function L(e) {
			I(e), a.push(e);
		}
		function R(e) {
			let t = a.indexOf(e);
			t !== -1 && a.splice(t, 1);
		}
		function z(e, t) {
			let n = e;
			a.forEach(function(e) {
				e[n] && e[n](t);
			});
		}
		function B(e) {
			return ue("10.7.0", "highlightBlock will be removed entirely in v12.0"), ue("10.7.0", "Please use highlightElement now."), w(e);
		}
		Object.assign(e, {
			highlight: p,
			highlightAuto: S,
			highlightAll: k,
			highlightElement: w,
			highlightBlock: B,
			configure: T,
			initHighlighting: E,
			initHighlightingOnLoad: D,
			registerLanguage: A,
			unregisterLanguage: j,
			listLanguages: M,
			getLanguage: N,
			registerAliases: P,
			autoDetection: F,
			inherit: Ce,
			addPlugin: L,
			removePlugin: R
		}), e.debugMode = function() {
			o = !1;
		}, e.safeMode = function() {
			o = !0;
		}, e.versionString = be, e.regex = {
			concat: _,
			lookahead: m,
			either: y,
			optional: g,
			anyNumberOfTimes: h
		};
		for (let e in G) typeof G[e] == "object" && n(G[e]);
		return Object.assign(e, G), e;
	}, De = Ee({});
	De.newInstance = () => Ee({}), t.exports = De, De.HighlightJS = De, De.default = De;
})))())).default;
//#endregion
//#region node_modules/highlight.js/es/languages/rust.js
function Ze(e) {
	let t = e.regex, n = /(r#)?/, r = t.concat(n, e.UNDERSCORE_IDENT_RE), i = t.concat(n, e.IDENT_RE), a = {
		className: "title.function.invoke",
		relevance: 0,
		begin: t.concat(/\b/, /(?!let|for|while|if|else|match\b)/, i, t.lookahead(/\s*\(/))
	}, o = "([ui](8|16|32|64|128|size)|f(32|64))?", s = /* @__PURE__ */ "abstract.as.async.await.become.box.break.const.continue.crate.do.dyn.else.enum.extern.false.final.fn.for.if.impl.in.let.loop.macro.match.mod.move.mut.override.priv.pub.ref.return.self.Self.static.struct.super.trait.true.try.type.typeof.union.unsafe.unsized.use.virtual.where.while.yield".split("."), c = [
		"true",
		"false",
		"Some",
		"None",
		"Ok",
		"Err"
	], l = /* @__PURE__ */ "drop .Copy.Send.Sized.Sync.Drop.Fn.FnMut.FnOnce.ToOwned.Clone.Debug.PartialEq.PartialOrd.Eq.Ord.AsRef.AsMut.Into.From.Default.Iterator.Extend.IntoIterator.DoubleEndedIterator.ExactSizeIterator.SliceConcatExt.ToString.assert!.assert_eq!.bitflags!.bytes!.cfg!.col!.concat!.concat_idents!.debug_assert!.debug_assert_eq!.env!.eprintln!.panic!.file!.format!.format_args!.include_bytes!.include_str!.line!.local_data_key!.module_path!.option_env!.print!.println!.select!.stringify!.try!.unimplemented!.unreachable!.vec!.write!.writeln!.macro_rules!.assert_ne!.debug_assert_ne!".split("."), u = [
		"i8",
		"i16",
		"i32",
		"i64",
		"i128",
		"isize",
		"u8",
		"u16",
		"u32",
		"u64",
		"u128",
		"usize",
		"f32",
		"f64",
		"str",
		"char",
		"bool",
		"Box",
		"Option",
		"Result",
		"String",
		"Vec"
	];
	return {
		name: "Rust",
		aliases: ["rs"],
		keywords: {
			$pattern: e.IDENT_RE + "!?",
			type: u,
			keyword: s,
			literal: c,
			built_in: l
		},
		illegal: "</",
		contains: [
			e.C_LINE_COMMENT_MODE,
			e.COMMENT("/\\*", "\\*/", { contains: ["self"] }),
			e.inherit(e.QUOTE_STRING_MODE, {
				begin: /b?"/,
				illegal: null
			}),
			{
				className: "symbol",
				begin: /'[a-zA-Z_][a-zA-Z0-9_]*(?!')/
			},
			{
				scope: "string",
				variants: [{ begin: /b?r(#*)"(.|\n)*?"\1(?!#)/ }, {
					begin: /b?'/,
					end: /'/,
					contains: [{
						scope: "char.escape",
						match: /\\('|\w|x\w{2}|u\w{4}|U\w{8})/
					}]
				}]
			},
			{
				className: "number",
				variants: [
					{ begin: "\\b0b([01_]+)" + o },
					{ begin: "\\b0o([0-7_]+)" + o },
					{ begin: "\\b0x([A-Fa-f0-9_]+)" + o },
					{ begin: "\\b(\\d[\\d_]*(\\.[0-9_]+)?([eE][+-]?[0-9_]+)?)" + o }
				],
				relevance: 0
			},
			{
				begin: [
					/fn/,
					/\s+/,
					r
				],
				className: {
					1: "keyword",
					3: "title.function"
				}
			},
			{
				className: "meta",
				begin: "#!?\\[",
				end: "\\]",
				contains: [{
					className: "string",
					begin: /"/,
					end: /"/,
					contains: [e.BACKSLASH_ESCAPE]
				}]
			},
			{
				begin: [
					/let/,
					/\s+/,
					/(?:mut\s+)?/,
					r
				],
				className: {
					1: "keyword",
					3: "keyword",
					4: "variable"
				}
			},
			{
				begin: [
					/for/,
					/\s+/,
					r,
					/\s+/,
					/in/
				],
				className: {
					1: "keyword",
					3: "variable",
					5: "keyword"
				}
			},
			{
				begin: [
					/type/,
					/\s+/,
					r
				],
				className: {
					1: "keyword",
					3: "title.class"
				}
			},
			{
				begin: [
					/(?:trait|enum|struct|union|impl|for)/,
					/\s+/,
					r
				],
				className: {
					1: "keyword",
					3: "title.class"
				}
			},
			{
				begin: e.IDENT_RE + "::",
				keywords: {
					keyword: "Self",
					built_in: l,
					type: u
				}
			},
			{
				className: "punctuation",
				begin: "->"
			},
			a
		]
	};
}
//#endregion
//#region node_modules/highlight.js/es/languages/python.js
function Qe(e) {
	let t = e.regex, n = /[\p{XID_Start}_]\p{XID_Continue}*/u, r = /* @__PURE__ */ "and.as.assert.async.await.break.case.class.continue.def.del.elif.else.except.finally.for.from.global.if.import.in.is.lambda.match.nonlocal|10.not.or.pass.raise.return.try.while.with.yield".split("."), i = {
		$pattern: /[A-Za-z]\w+|__\w+__/,
		keyword: r,
		built_in: /* @__PURE__ */ "__import__.abs.all.any.ascii.bin.bool.breakpoint.bytearray.bytes.callable.chr.classmethod.compile.complex.delattr.dict.dir.divmod.enumerate.eval.exec.filter.float.format.frozenset.getattr.globals.hasattr.hash.help.hex.id.input.int.isinstance.issubclass.iter.len.list.locals.map.max.memoryview.min.next.object.oct.open.ord.pow.print.property.range.repr.reversed.round.set.setattr.slice.sorted.staticmethod.str.sum.super.tuple.type.vars.zip".split("."),
		literal: [
			"__debug__",
			"Ellipsis",
			"False",
			"None",
			"NotImplemented",
			"True"
		],
		type: [
			"Any",
			"Callable",
			"Coroutine",
			"Dict",
			"List",
			"Literal",
			"Generic",
			"Optional",
			"Sequence",
			"Set",
			"Tuple",
			"Type",
			"Union"
		]
	}, a = {
		className: "meta",
		begin: /^(>>>|\.\.\.) /
	}, o = {
		className: "subst",
		begin: /\{/,
		end: /\}/,
		keywords: i,
		illegal: /#/
	}, s = {
		begin: /\{\{/,
		relevance: 0
	}, c = {
		className: "string",
		contains: [e.BACKSLASH_ESCAPE],
		variants: [
			{
				begin: /([uU]|[bB]|[rR]|[bB][rR]|[rR][bB])?'''/,
				end: /'''/,
				contains: [e.BACKSLASH_ESCAPE, a],
				relevance: 10
			},
			{
				begin: /([uU]|[bB]|[rR]|[bB][rR]|[rR][bB])?"""/,
				end: /"""/,
				contains: [e.BACKSLASH_ESCAPE, a],
				relevance: 10
			},
			{
				begin: /([fF][rR]|[rR][fF]|[fF])'''/,
				end: /'''/,
				contains: [
					e.BACKSLASH_ESCAPE,
					a,
					s,
					o
				]
			},
			{
				begin: /([fF][rR]|[rR][fF]|[fF])"""/,
				end: /"""/,
				contains: [
					e.BACKSLASH_ESCAPE,
					a,
					s,
					o
				]
			},
			{
				begin: /([uU]|[rR])'/,
				end: /'/,
				relevance: 10
			},
			{
				begin: /([uU]|[rR])"/,
				end: /"/,
				relevance: 10
			},
			{
				begin: /([bB]|[bB][rR]|[rR][bB])'/,
				end: /'/
			},
			{
				begin: /([bB]|[bB][rR]|[rR][bB])"/,
				end: /"/
			},
			{
				begin: /([fF][rR]|[rR][fF]|[fF])'/,
				end: /'/,
				contains: [
					e.BACKSLASH_ESCAPE,
					s,
					o
				]
			},
			{
				begin: /([fF][rR]|[rR][fF]|[fF])"/,
				end: /"/,
				contains: [
					e.BACKSLASH_ESCAPE,
					s,
					o
				]
			},
			e.APOS_STRING_MODE,
			e.QUOTE_STRING_MODE
		]
	}, l = "[0-9](_?[0-9])*", u = `(\\b(${l}))?\\.(${l})|\\b(${l})\\.`, d = `\\b|${r.join("|")}`, f = {
		className: "number",
		relevance: 0,
		variants: [
			{ begin: `(\\b(${l})|(${u}))[eE][+-]?(${l})[jJ]?(?=${d})` },
			{ begin: `(${u})[jJ]?` },
			{ begin: `\\b([1-9](_?[0-9])*|0+(_?0)*)[lLjJ]?(?=${d})` },
			{ begin: `\\b0[bB](_?[01])+[lL]?(?=${d})` },
			{ begin: `\\b0[oO](_?[0-7])+[lL]?(?=${d})` },
			{ begin: `\\b0[xX](_?[0-9a-fA-F])+[lL]?(?=${d})` },
			{ begin: `\\b(${l})[jJ](?=${d})` }
		]
	}, p = {
		className: "comment",
		begin: t.lookahead(/# type:/),
		end: /$/,
		keywords: i,
		contains: [{ begin: /# type:/ }, {
			begin: /#/,
			end: /\b\B/,
			endsWithParent: !0
		}]
	}, m = {
		className: "params",
		variants: [{
			className: "",
			begin: /\(\s*\)/,
			skip: !0
		}, {
			begin: /\(/,
			end: /\)/,
			excludeBegin: !0,
			excludeEnd: !0,
			keywords: i,
			contains: [
				"self",
				a,
				f,
				c,
				e.HASH_COMMENT_MODE
			]
		}]
	};
	return o.contains = [
		c,
		f,
		a
	], {
		name: "Python",
		aliases: [
			"py",
			"gyp",
			"ipython"
		],
		unicodeRegex: !0,
		keywords: i,
		illegal: /(<\/|\?)|=>/,
		contains: [
			a,
			f,
			{
				scope: "variable.language",
				match: /\bself\b/
			},
			{
				beginKeywords: "if",
				relevance: 0
			},
			{
				match: /\bor\b/,
				scope: "keyword"
			},
			c,
			p,
			e.HASH_COMMENT_MODE,
			{
				match: [
					/\bdef/,
					/\s+/,
					n
				],
				scope: {
					1: "keyword",
					3: "title.function"
				},
				contains: [m]
			},
			{
				variants: [{ match: [
					/\bclass/,
					/\s+/,
					n,
					/\s*/,
					/\(\s*/,
					n,
					/\s*\)/
				] }, { match: [
					/\bclass/,
					/\s+/,
					n
				] }],
				scope: {
					1: "keyword",
					3: "title.class",
					6: "title.class.inherited"
				}
			},
			{
				className: "meta",
				begin: /^[\t ]*@/,
				end: /(?=#)|$/,
				contains: [
					f,
					m,
					c
				]
			}
		]
	};
}
//#endregion
//#region node_modules/highlight.js/es/languages/typescript.js
var $e = "[A-Za-z$_][0-9A-Za-z$_]*", et = /* @__PURE__ */ "as.in.of.if.for.while.finally.var.new.function.do.return.void.else.break.catch.instanceof.with.throw.case.default.try.switch.continue.typeof.delete.let.yield.const.class.debugger.async.await.static.import.from.export.extends.using".split("."), tt = [
	"true",
	"false",
	"null",
	"undefined",
	"NaN",
	"Infinity"
], nt = /* @__PURE__ */ "Object.Function.Boolean.Symbol.Math.Date.Number.BigInt.String.RegExp.Array.Float32Array.Float64Array.Int8Array.Uint8Array.Uint8ClampedArray.Int16Array.Int32Array.Uint16Array.Uint32Array.BigInt64Array.BigUint64Array.Set.Map.WeakSet.WeakMap.ArrayBuffer.SharedArrayBuffer.Atomics.DataView.JSON.Promise.Generator.GeneratorFunction.AsyncFunction.Reflect.Proxy.Intl.WebAssembly".split("."), rt = [
	"Error",
	"EvalError",
	"InternalError",
	"RangeError",
	"ReferenceError",
	"SyntaxError",
	"TypeError",
	"URIError"
], it = [
	"setInterval",
	"setTimeout",
	"clearInterval",
	"clearTimeout",
	"require",
	"exports",
	"eval",
	"isFinite",
	"isNaN",
	"parseFloat",
	"parseInt",
	"decodeURI",
	"decodeURIComponent",
	"encodeURI",
	"encodeURIComponent",
	"escape",
	"unescape"
], at = [
	"arguments",
	"this",
	"super",
	"console",
	"window",
	"document",
	"localStorage",
	"sessionStorage",
	"module",
	"global"
], ot = [].concat(it, nt, rt);
function st(e) {
	let t = e.regex, n = (e, { after: t }) => {
		let n = "</" + e[0].slice(1);
		return e.input.indexOf(n, t) !== -1;
	}, r = $e, i = {
		begin: "<>",
		end: "</>"
	}, a = /<[A-Za-z0-9\\._:-]+\s*\/>/, o = {
		begin: /<[A-Za-z0-9\\._:-]+/,
		end: /\/[A-Za-z0-9\\._:-]+>|\/>/,
		isTrulyOpeningTag: (e, t) => {
			let r = e[0].length + e.index, i = e.input[r];
			if (i === "<" || i === ",") {
				t.ignoreMatch();
				return;
			}
			i === ">" && (n(e, { after: r }) || t.ignoreMatch());
			let a, o = e.input.substring(r);
			if (a = o.match(/^\s*=/)) {
				t.ignoreMatch();
				return;
			}
			if ((a = o.match(/^\s+extends\s+/)) && a.index === 0) {
				t.ignoreMatch();
				return;
			}
		}
	}, s = {
		$pattern: $e,
		keyword: et,
		literal: tt,
		built_in: ot,
		"variable.language": at
	}, c = "[0-9](_?[0-9])*", l = `\\.(${c})`, u = "0|[1-9](_?[0-9])*|0[0-7]*[89][0-9]*", d = {
		className: "number",
		variants: [
			{ begin: `(\\b(${u})((${l})|\\.)?|(${l}))[eE][+-]?(${c})\\b` },
			{ begin: `\\b(${u})\\b((${l})\\b|\\.)?|(${l})\\b` },
			{ begin: "\\b(0|[1-9](_?[0-9])*)n\\b" },
			{ begin: "\\b0[xX][0-9a-fA-F](_?[0-9a-fA-F])*n?\\b" },
			{ begin: "\\b0[bB][0-1](_?[0-1])*n?\\b" },
			{ begin: "\\b0[oO][0-7](_?[0-7])*n?\\b" },
			{ begin: "\\b0[0-7]+n?\\b" }
		],
		relevance: 0
	}, f = {
		className: "subst",
		begin: "\\$\\{",
		end: "\\}",
		keywords: s,
		contains: []
	}, p = {
		begin: ".?html`",
		end: "",
		starts: {
			end: "`",
			returnEnd: !1,
			contains: [e.BACKSLASH_ESCAPE, f],
			subLanguage: "xml"
		}
	}, m = {
		begin: ".?css`",
		end: "",
		starts: {
			end: "`",
			returnEnd: !1,
			contains: [e.BACKSLASH_ESCAPE, f],
			subLanguage: "css"
		}
	}, h = {
		begin: ".?gql`",
		end: "",
		starts: {
			end: "`",
			returnEnd: !1,
			contains: [e.BACKSLASH_ESCAPE, f],
			subLanguage: "graphql"
		}
	}, g = {
		className: "string",
		begin: "`",
		end: "`",
		contains: [e.BACKSLASH_ESCAPE, f]
	}, _ = {
		className: "comment",
		variants: [
			e.COMMENT(/\/\*\*(?!\/)/, "\\*/", {
				relevance: 0,
				contains: [{
					begin: "(?=@[A-Za-z]+)",
					relevance: 0,
					contains: [
						{
							className: "doctag",
							begin: "@[A-Za-z]+"
						},
						{
							className: "type",
							begin: "\\{",
							end: "\\}",
							excludeEnd: !0,
							excludeBegin: !0,
							relevance: 0
						},
						{
							className: "variable",
							begin: r + "(?=\\s*(-)|$)",
							endsParent: !0,
							relevance: 0
						},
						{
							begin: /(?=[^\n])\s/,
							relevance: 0
						}
					]
				}]
			}),
			e.C_BLOCK_COMMENT_MODE,
			e.C_LINE_COMMENT_MODE
		]
	}, v = [
		e.APOS_STRING_MODE,
		e.QUOTE_STRING_MODE,
		p,
		m,
		h,
		g,
		{ match: /\$\d+/ },
		d
	];
	f.contains = v.concat({
		begin: /\{/,
		end: /\}/,
		keywords: s,
		contains: ["self"].concat(v)
	});
	let y = [].concat(_, f.contains), b = y.concat([{
		begin: /(\s*)\(/,
		end: /\)/,
		keywords: s,
		contains: ["self"].concat(y)
	}]), x = {
		className: "params",
		begin: /(\s*)\(/,
		end: /\)/,
		excludeBegin: !0,
		excludeEnd: !0,
		keywords: s,
		contains: b
	}, S = { variants: [{
		match: [
			/class/,
			/\s+/,
			r,
			/\s+/,
			/extends/,
			/\s+/,
			t.concat(r, "(", t.concat(/\./, r), ")*")
		],
		scope: {
			1: "keyword",
			3: "title.class",
			5: "keyword",
			7: "title.class.inherited"
		}
	}, {
		match: [
			/class/,
			/\s+/,
			r
		],
		scope: {
			1: "keyword",
			3: "title.class"
		}
	}] }, C = {
		relevance: 0,
		match: t.either(/\bJSON/, /\b[A-Z][a-z]+([A-Z][a-z]*|\d)*/, /\b[A-Z]{2,}([A-Z][a-z]+|\d)+([A-Z][a-z]*)*/, /\b[A-Z]{2,}[a-z]+([A-Z][a-z]+|\d)*([A-Z][a-z]*)*/),
		className: "title.class",
		keywords: { _: [...nt, ...rt] }
	}, w = {
		label: "use_strict",
		className: "meta",
		relevance: 10,
		begin: /^\s*['"]use (strict|asm)['"]/
	}, T = {
		variants: [{ match: [
			/function/,
			/\s+/,
			r,
			/(?=\s*\()/
		] }, { match: [/function/, /\s*(?=\()/] }],
		className: {
			1: "keyword",
			3: "title.function"
		},
		label: "func.def",
		contains: [x],
		illegal: /%/
	}, E = {
		relevance: 0,
		match: /\b[A-Z][A-Z_0-9]+\b/,
		className: "variable.constant"
	};
	function D(e) {
		return t.concat("(?!", e.join("|"), ")");
	}
	let O = {
		match: t.concat(/\b/, D([
			...it,
			"super",
			"import"
		].map((e) => `${e}\\s*\\(`)), r, t.lookahead(/\s*\(/)),
		className: "title.function",
		relevance: 0
	}, k = {
		begin: t.concat(/\./, t.lookahead(t.concat(r, /(?![0-9A-Za-z$_(])/))),
		end: r,
		excludeBegin: !0,
		keywords: "prototype",
		className: "property",
		relevance: 0
	}, A = {
		match: [
			/get|set/,
			/\s+/,
			r,
			/(?=\()/
		],
		className: {
			1: "keyword",
			3: "title.function"
		},
		contains: [{ begin: /\(\)/ }, x]
	}, j = "(\\([^()]*(\\([^()]*(\\([^()]*\\)[^()]*)*\\)[^()]*)*\\)|" + e.UNDERSCORE_IDENT_RE + ")\\s*=>", M = {
		match: [
			/const|var|let/,
			/\s+/,
			r,
			/\s*/,
			/=\s*/,
			/(async\s*)?/,
			t.lookahead(j)
		],
		keywords: "async",
		className: {
			1: "keyword",
			3: "title.function"
		},
		contains: [x]
	};
	return {
		name: "JavaScript",
		aliases: [
			"js",
			"jsx",
			"mjs",
			"cjs"
		],
		keywords: s,
		exports: {
			PARAMS_CONTAINS: b,
			CLASS_REFERENCE: C
		},
		illegal: /#(?![$_A-z])/,
		contains: [
			e.SHEBANG({
				label: "shebang",
				binary: "node",
				relevance: 5
			}),
			w,
			e.APOS_STRING_MODE,
			e.QUOTE_STRING_MODE,
			p,
			m,
			h,
			g,
			_,
			{ match: /\$\d+/ },
			d,
			C,
			{
				scope: "attr",
				match: r + t.lookahead(":"),
				relevance: 0
			},
			M,
			{
				begin: "(" + e.RE_STARTERS_RE + "|\\b(case|return|throw)\\b)\\s*",
				keywords: "return throw case",
				relevance: 0,
				contains: [
					_,
					e.REGEXP_MODE,
					{
						className: "function",
						begin: j,
						returnBegin: !0,
						end: "\\s*=>",
						contains: [{
							className: "params",
							variants: [
								{
									begin: e.UNDERSCORE_IDENT_RE,
									relevance: 0
								},
								{
									className: null,
									begin: /\(\s*\)/,
									skip: !0
								},
								{
									begin: /(\s*)\(/,
									end: /\)/,
									excludeBegin: !0,
									excludeEnd: !0,
									keywords: s,
									contains: b
								}
							]
						}]
					},
					{
						begin: /,/,
						relevance: 0
					},
					{
						match: /\s+/,
						relevance: 0
					},
					{
						variants: [
							{
								begin: i.begin,
								end: i.end
							},
							{ match: a },
							{
								begin: o.begin,
								"on:begin": o.isTrulyOpeningTag,
								end: o.end
							}
						],
						subLanguage: "xml",
						contains: [{
							begin: o.begin,
							end: o.end,
							skip: !0,
							contains: ["self"]
						}]
					}
				]
			},
			T,
			{ beginKeywords: "while if switch catch for" },
			{
				begin: "\\b(?!function)" + e.UNDERSCORE_IDENT_RE + "\\([^()]*(\\([^()]*(\\([^()]*\\)[^()]*)*\\)[^()]*)*\\)\\s*\\{",
				returnBegin: !0,
				label: "func.def",
				contains: [x, e.inherit(e.TITLE_MODE, {
					begin: r,
					className: "title.function"
				})]
			},
			{
				match: /\.\.\./,
				relevance: 0
			},
			k,
			{
				match: "\\$" + r,
				relevance: 0
			},
			{
				match: [/\bconstructor(?=\s*\()/],
				className: { 1: "title.function" },
				contains: [x]
			},
			O,
			E,
			S,
			A,
			{ match: /\$[(.]/ }
		]
	};
}
function ct(e) {
	let t = e.regex, n = st(e), r = $e, i = [
		"any",
		"void",
		"number",
		"boolean",
		"string",
		"object",
		"never",
		"symbol",
		"bigint",
		"unknown"
	], a = {
		begin: [
			/namespace/,
			/\s+/,
			e.IDENT_RE
		],
		beginScope: {
			1: "keyword",
			3: "title.class"
		}
	}, o = {
		beginKeywords: "interface",
		end: /\{/,
		excludeEnd: !0,
		keywords: {
			keyword: "interface extends",
			built_in: i
		},
		contains: [n.exports.CLASS_REFERENCE]
	}, s = {
		className: "meta",
		relevance: 10,
		begin: /^\s*['"]use strict['"]/
	}, c = {
		$pattern: $e,
		keyword: et.concat([
			"type",
			"interface",
			"public",
			"private",
			"protected",
			"implements",
			"declare",
			"abstract",
			"readonly",
			"enum",
			"override",
			"satisfies"
		]),
		literal: tt,
		built_in: ot.concat(i),
		"variable.language": at
	}, l = {
		className: "meta",
		begin: "@" + r
	}, u = (e, t, n) => {
		let r = e.contains.findIndex((e) => e.label === t);
		if (r === -1) throw Error("can not find mode to replace");
		e.contains.splice(r, 1, n);
	};
	Object.assign(n.keywords, c), n.exports.PARAMS_CONTAINS.push(l);
	let d = n.contains.find((e) => e.scope === "attr"), f = Object.assign({}, d, { match: t.concat(r, t.lookahead(/\s*\?:/)) });
	n.exports.PARAMS_CONTAINS.push([
		n.exports.CLASS_REFERENCE,
		d,
		f
	]), n.contains = n.contains.concat([
		l,
		a,
		o,
		f
	]), u(n, "shebang", e.SHEBANG()), u(n, "use_strict", s);
	let p = n.contains.find((e) => e.label === "func.def");
	return p.relevance = 0, Object.assign(n, {
		name: "TypeScript",
		aliases: [
			"ts",
			"tsx",
			"mts",
			"cts"
		]
	}), n;
}
//#endregion
//#region node_modules/highlight.js/es/languages/c.js
function lt(e) {
	let t = e.regex, n = e.COMMENT("//", "$", { contains: [{ begin: /\\\n/ }] }), r = "decltype\\(auto\\)", i = "[a-zA-Z_]\\w*::", a = "(" + r + "|" + t.optional(i) + "[a-zA-Z_]\\w*" + t.optional("<[^<>]+>") + ")", o = {
		className: "type",
		variants: [{ begin: "\\b[a-z\\d_]*_t\\b" }, { match: /\batomic_[a-z]{3,6}\b/ }]
	}, s = {
		className: "string",
		variants: [
			{
				begin: "(u8?|U|L)?\"",
				end: "\"",
				illegal: "\\n",
				contains: [e.BACKSLASH_ESCAPE]
			},
			{
				begin: "(u8?|U|L)?'(\\\\(x[0-9A-Fa-f]{2}|u[0-9A-Fa-f]{4,8}|[0-7]{3}|\\S)|.)",
				end: "'",
				illegal: "."
			},
			e.END_SAME_AS_BEGIN({
				begin: /(?:u8?|U|L)?R"([^()\\ ]{0,16})\(/,
				end: /\)([^()\\ ]{0,16})"/
			})
		]
	}, c = {
		className: "number",
		variants: [
			{ match: /\b(0b[01']+)/ },
			{ match: /(-?)\b([\d']+(\.[\d']*)?|\.[\d']+)((ll|LL|l|L)(u|U)?|(u|U)(ll|LL|l|L)?|f|F|b|B)/ },
			{ match: /(-?)\b(0[xX][a-fA-F0-9]+(?:'[a-fA-F0-9]+)*(?:\.[a-fA-F0-9]*(?:'[a-fA-F0-9]*)*)?(?:[pP][-+]?[0-9]+)?(l|L)?(u|U)?)/ },
			{ match: /(-?)\b\d+(?:'\d+)*(?:\.\d*(?:'\d*)*)?(?:[eE][-+]?\d+)?/ }
		],
		relevance: 0
	}, l = {
		className: "meta",
		begin: /#\s*[a-z]+\b/,
		end: /$/,
		keywords: { keyword: "if else elif endif define undef warning error line pragma _Pragma ifdef ifndef elifdef elifndef include" },
		contains: [
			{
				begin: /\\\n/,
				relevance: 0
			},
			e.inherit(s, { className: "string" }),
			{
				className: "string",
				begin: /<.*?>/
			},
			n,
			e.C_BLOCK_COMMENT_MODE
		]
	}, u = {
		className: "title",
		begin: t.optional(i) + e.IDENT_RE,
		relevance: 0
	}, d = t.optional(i) + e.IDENT_RE + "\\s*\\(", f = {
		keyword: /* @__PURE__ */ "asm.auto.break.case.continue.default.do.else.enum.extern.for.fortran.goto.if.inline.register.restrict.return.sizeof.typeof.typeof_unqual.struct.switch.typedef.union.volatile.while._Alignas._Alignof._Atomic._Generic._Noreturn._Static_assert._Thread_local.alignas.alignof.noreturn.static_assert.thread_local._Pragma".split("."),
		type: /* @__PURE__ */ "float.double.signed.unsigned.int.short.long.char.void._Bool._BitInt._Complex._Imaginary._Decimal32._Decimal64._Decimal96._Decimal128._Decimal64x._Decimal128x._Float16._Float32._Float64._Float128._Float32x._Float64x._Float128x.const.static.constexpr.complex.bool.imaginary".split("."),
		literal: "true false NULL",
		built_in: "std string wstring cin cout cerr clog stdin stdout stderr stringstream istringstream ostringstream auto_ptr deque list queue stack vector map set pair bitset multiset multimap unordered_set unordered_map unordered_multiset unordered_multimap priority_queue make_pair array shared_ptr abort terminate abs acos asin atan2 atan calloc ceil cosh cos exit exp fabs floor fmod fprintf fputs free frexp fscanf future isalnum isalpha iscntrl isdigit isgraph islower isprint ispunct isspace isupper isxdigit tolower toupper labs ldexp log10 log malloc realloc memchr memcmp memcpy memset modf pow printf putchar puts scanf sinh sin snprintf sprintf sqrt sscanf strcat strchr strcmp strcpy strcspn strlen strncat strncmp strncpy strpbrk strrchr strspn strstr tanh tan vfprintf vprintf vsprintf endl initializer_list unique_ptr"
	}, p = [
		l,
		o,
		n,
		e.C_BLOCK_COMMENT_MODE,
		c,
		s
	], m = {
		variants: [
			{
				begin: /=/,
				end: /;/
			},
			{
				begin: /\(/,
				end: /\)/
			},
			{
				beginKeywords: "new throw return else",
				end: /;/
			}
		],
		keywords: f,
		contains: p.concat([{
			begin: /\(/,
			end: /\)/,
			keywords: f,
			contains: p.concat(["self"]),
			relevance: 0
		}]),
		relevance: 0
	}, h = {
		begin: "(" + a + "[\\*&\\s]+)+" + d,
		returnBegin: !0,
		end: /[{;=]/,
		excludeEnd: !0,
		keywords: f,
		illegal: /[^\w\s\*&:<>.]/,
		contains: [
			{
				begin: r,
				keywords: f,
				relevance: 0
			},
			{
				begin: d,
				returnBegin: !0,
				contains: [e.inherit(u, { className: "title.function" })],
				relevance: 0
			},
			{
				relevance: 0,
				match: /,/
			},
			{
				className: "params",
				begin: /\(/,
				end: /\)/,
				keywords: f,
				relevance: 0,
				contains: [
					n,
					e.C_BLOCK_COMMENT_MODE,
					s,
					c,
					o,
					{
						begin: /\(/,
						end: /\)/,
						keywords: f,
						relevance: 0,
						contains: [
							"self",
							n,
							e.C_BLOCK_COMMENT_MODE,
							s,
							c,
							o
						]
					}
				]
			},
			o,
			n,
			e.C_BLOCK_COMMENT_MODE,
			l
		]
	};
	return {
		name: "C",
		aliases: ["h"],
		keywords: f,
		disableAutodetect: !0,
		illegal: "</",
		contains: [].concat(m, h, p, [
			l,
			{
				begin: e.IDENT_RE + "::",
				keywords: f
			},
			{
				className: "class",
				beginKeywords: "enum class struct union",
				end: /[{;:<>=]/,
				contains: [{ beginKeywords: "final class struct" }, e.TITLE_MODE]
			}
		]),
		exports: {
			preprocessor: l,
			strings: s,
			keywords: f
		}
	};
}
//#endregion
//#region src/lang/abt.ts
var ut = () => ({
	name: "ABT",
	aliases: ["abt"],
	case_insensitive: !1,
	contains: [
		{
			className: "comment",
			begin: /;|#/,
			end: /$/
		},
		{
			className: "section",
			begin: /^\s*\.(strings|exports|code|object_keys|object_types|line)\b/,
			relevance: 10
		},
		{
			className: "label",
			begin: /^\s*[A-Za-z_][A-Za-z0-9_]*:/,
			relevance: 5
		},
		{
			className: "link",
			begin: /@[A-Za-z_][A-Za-z0-9_]*/,
			relevance: 3
		},
		{
			className: "variable",
			begin: /\barg\d+\b/,
			relevance: 2
		},
		{
			className: "keyword",
			begin: RegExp(`\\b(${(/* @__PURE__ */ "nop,pop,pop.n,dup,swap,drop,reserve,const.i32,const.u8,const.0,const.1,const.f32,const.f64,const.i64,const.u64,load.str,set.field,set.elem,get.elem,get.field,create.obj,create.arr,arr.len,mod.f,mod.d,slice,create.tuple,get.tuple.field,promote.f64,ret.d,create.range,create.range.eq,build.fstr,null.coalesce,error.propagate,create.node,create.some,create.none,create.ok,create.err,is.some,is.ok,unwrap.some,unwrap.ok,unwrap.err,cast.i32,cast.u32,cast.i64,cast.u64,cast.f64,cast.ptr,to.str,to.i32,to.f64,f64.to.str,i64.to.str,u64.to.str,bool.to.str,f64.to.i32,str.to.i64,f32.to.str,f32.to.i32,to_str,is.nil,str.cat,load.local,store.local,load.loc.0,load.loc.1,load.loc.2,store.loc.0,store.loc.1,add,sub,mul,div,mod,neg,add.f,sub.f,mul.f,div.f,neg.f,add.d,sub.d,mul.d,div.d,neg.d,mod.u64,i32.to.f32,i64.to.f64,u64.to.f64,add.u64,sub.u64,mul.u64,div.u64,and,or,xor,not,shl,shr,eq,ne,lt,gt,le,ge,eq.d,ne.d,lt.d,gt.d,le.d,ge.d,jmp,jmp.z,jmp.nz,jmp.l,call,ret,call.nat,call.spec,spawn,task.id,yield,sleep,join,chan.new,send,recv,try.recv,spawn.go,task.loop,handle.msg,reply,closure,capture.var,load.captured,store.captured,call.closure,create.list.int,create.list.str,create.list.bool,list.push.int,list.pop.int,list.get.int,list.set.int,create.list.int.inline,create.list.str.inline,create.list.bool.inline,new.instance,construct.instance,get.generic.field,set.generic.field,load.ref,store.ref,load.mut.ref,store.mut.ref,fn.prolog,is.variant,create.future,await.future,poll.future,.line,print,halt".split(",")).join("|")})\\b`),
			relevance: 1
		},
		{
			className: "string",
			begin: /\bstr\[/,
			end: /\]/,
			relevance: 1
		},
		{
			className: "string",
			begin: /\bfield\[/,
			end: /\]/,
			relevance: 1
		},
		{
			className: "number",
			begin: /\bnat#\d+\b/,
			relevance: 1
		},
		{
			className: "number",
			begin: /\b0x[0-9a-fA-F]+\b/,
			relevance: 0
		},
		{
			className: "number",
			begin: /\b-?\d+\.?\d*\b/,
			relevance: 0
		},
		{
			className: "string",
			begin: /"/,
			end: /"/,
			contains: [{
				className: "subst",
				begin: /\\./
			}]
		}
	]
}), dt = { class: "lines-container" }, ft = ["onClick"], pt = { class: "line-number" }, mt = ["innerHTML"], ht = {
	key: 0,
	class: "code-line"
}, gt = /* @__PURE__ */ $(/* @__PURE__ */ l({
	__name: "CodePreview",
	props: {
		code: {},
		language: {},
		highlightLines: {}
	},
	emits: ["line-click"],
	setup(o, { emit: s }) {
		Xe.registerLanguage("rust", Ze), Xe.registerLanguage("python", Qe), Xe.registerLanguage("typescript", ct), Xe.registerLanguage("c", lt), Xe.registerLanguage("abt", ut);
		let c = o, l = s, u = {
			rust: "rust",
			python: "python",
			typescript: "typescript",
			c: "c",
			abt: "abt"
		}, d = t(() => {
			if (!c.code) return [""];
			let e = c.language ? u[c.language] : void 0;
			if (!e) return c.code.split("\n");
			try {
				return Xe.highlight(c.code, { language: e }).value.split("\n");
			} catch {
				return c.code.split("\n");
			}
		});
		function f(e) {
			return c.highlightLines?.includes(e) ?? !1;
		}
		function m(e) {
			l("line-click", e);
		}
		return (t, o) => (v(), n(Ye, { class: "code-preview" }, {
			default: k(() => [a("div", dt, [(v(!0), i(e, null, b(d.value, (e, t) => (v(), i("div", {
				key: t,
				class: p(["code-line", { highlighted: f(t + 1) }]),
				onClick: (e) => m(t + 1)
			}, [a("span", pt, C(t + 1), 1), a("span", {
				class: "line-content",
				innerHTML: e || " "
			}, null, 8, mt)], 10, ft))), 128)), d.value.length === 0 ? (v(), i("div", ht, [...o[0] ||= [a("span", { class: "line-number" }, "1", -1), a("span", { class: "line-content" }, null, -1)]])) : r("", !0)])]),
			_: 1
		}));
	}
}), [["__scopeId", "data-v-07e5fb27"]]), _t = {
	key: 0,
	class: "time-info"
}, vt = {
	key: 1,
	class: "stdout"
}, yt = {
	key: 2,
	class: "stderr"
}, bt = {
	key: 3,
	class: "result"
}, xt = {
	key: 4,
	class: "empty"
}, St = /* @__PURE__ */ $(/* @__PURE__ */ l({
	__name: "ConsoleOutput",
	props: {
		stdout: {},
		stderr: {},
		result: {},
		timeMs: {}
	},
	setup(e) {
		return (t, a) => (v(), n(Ye, { class: "console-output" }, {
			default: k(() => [
				e.timeMs > 0 ? (v(), i("div", _t, "Completed in " + C(e.timeMs) + "ms", 1)) : r("", !0),
				e.stdout ? (v(), i("pre", vt, C(e.stdout), 1)) : r("", !0),
				e.stderr ? (v(), i("pre", yt, C(e.stderr), 1)) : r("", !0),
				e.result ? (v(), i("pre", bt, "Result: " + C(e.result), 1)) : r("", !0),
				!e.stdout && !e.stderr && !e.result ? (v(), i("div", xt, "Click Run or press Ctrl+Enter to execute")) : r("", !0)
			]),
			_: 1
		}));
	}
}), [["__scopeId", "data-v-722a3db0"]]), Ct = !1, wt = null;
function Tt() {
	return Ct ? Promise.resolve() : wt || (wt = new Promise((e, t) => {
		if (window.ts) {
			Ct = !0, e();
			return;
		}
		let n = document.createElement("script");
		n.src = "https://cdn.jsdelivr.net/npm/typescript@5.7.3/lib/typescript.js", n.onload = () => {
			Ct = !0, e();
		}, n.onerror = () => t(/* @__PURE__ */ Error("Failed to load TypeScript compiler")), document.head.appendChild(n);
	}), wt);
}
async function Et(e) {
	try {
		await Tt();
	} catch (e) {
		return {
			stdout: "",
			stderr: `Failed to load TypeScript compiler: ${e}`
		};
	}
	let t = window.ts;
	if (!t) return {
		stdout: "",
		stderr: "TypeScript compiler not available"
	};
	let n;
	try {
		n = t.transpileModule(e, { compilerOptions: {
			module: t.ModuleKind.ES2015,
			target: t.ScriptTarget.ES2015,
			removeComments: !0
		} }).outputText;
	} catch (e) {
		return {
			stdout: "",
			stderr: `TypeScript compilation error: ${e}`
		};
	}
	let r = document.createElement("iframe");
	r.style.display = "none", document.body.appendChild(r);
	let i = [], a = [];
	try {
		let e = r.contentWindow, t = r.contentDocument, o = e;
		o.console.log = (...e) => {
			i.push(e.map((e) => String(e)).join(" "));
		}, o.console.error = (...e) => {
			a.push(e.map((e) => String(e)).join(" "));
		}, o.console.warn = (...e) => {
			a.push(e.map((e) => String(e)).join(" "));
		}, o.console.info = (...e) => {
			i.push(e.map((e) => String(e)).join(" "));
		};
		let s = t.createElement("script");
		s.textContent = n, t.body.appendChild(s);
	} catch (e) {
		a.push(String(e));
	} finally {
		setTimeout(() => {
			r.parentNode && document.body.removeChild(r);
		}, 100);
	}
	return {
		stdout: i.join("\n"),
		stderr: a.join("\n")
	};
}
//#endregion
//#region src/composables/usePlayground.ts
var Dt = 500, Ot = "// Welcome to Auto Playground!\nfn add(a int, b int) int {\n    a + b\n}\n\nlet result = add(3, 4)\nprint(result)";
function kt(e = {}) {
	let n = e.apiBase ?? "/api", r = e.persistKey ?? "auto-playground:state", i = e.defaultSource ?? Ot, a = e.preloadTargets ?? !0;
	function o() {
		if (typeof window > "u") return {};
		let e = window.location.hash;
		if (e.startsWith("#share=")) try {
			let t = atob(decodeURIComponent(e.slice(7))), n = JSON.parse(t);
			if (n.source) return n;
		} catch {}
		if (r === !1) return {};
		try {
			let e = localStorage.getItem(r);
			if (e) return JSON.parse(e);
		} catch {}
		return {};
	}
	function s(e) {
		if (!(typeof window > "u") && r !== !1) try {
			localStorage.setItem(r, JSON.stringify(e));
		} catch {}
	}
	let c = o(), l = y(c.source ?? i), u = y(""), d = y(""), f = y(""), p = y(0), m = y([]), h = y(!1), g = y(c.activeTab ?? "rust"), _ = y(""), v = y(c.liveCompile ?? !0), b = y(c.projectDir), x = y({}), S = y(null), C = y([]), w = y([]), T = y({
		message: "",
		visible: !1
	}), E = null, D = t(() => {
		let e = _.value;
		if (!e) return "";
		let t = x.value[e];
		return t ? t.files.find((e) => e.path === t.selectedFile)?.code ?? t.files[0]?.code ?? "" : "";
	}), k = t(() => {
		let e = _.value;
		return e ? x.value[e]?.files ?? [] : [];
	}), A = t(() => {
		let e = _.value;
		return e ? x.value[e]?.selectedFile ?? "" : "";
	}), j = t(() => ""), M = t(() => {
		let e = _.value, t = /* @__PURE__ */ new Map();
		if (!e) return t;
		let n = x.value[e];
		if (!n) return t;
		for (let e of n.files) {
			let r = n.fileSourceMaps[e.path] ?? [];
			for (let n of r) {
				let r = n.source_file || j.value;
				t.has(r) || t.set(r, /* @__PURE__ */ new Map());
				let i = t.get(r);
				i.has(n.source_line) || i.set(n.source_line, []);
				let a = i.get(n.source_line), o = a.find((t) => t.outputFile === e.path);
				o ? o.outputLines.includes(n.output_line) || o.outputLines.push(n.output_line) : a.push({
					outputFile: e.path,
					outputLines: [n.output_line]
				});
			}
		}
		return t;
	}), N = t(() => {
		let e = _.value, t = /* @__PURE__ */ new Map();
		if (!e) return t;
		let n = x.value[e];
		if (!n) return t;
		for (let e of n.files) {
			let r = n.fileSourceMaps[e.path] ?? [];
			for (let n of r) {
				let r = n.source_file || j.value;
				t.has(e.path) || t.set(e.path, /* @__PURE__ */ new Map()), t.get(e.path).set(n.output_line, {
					sourceFile: r,
					sourceLine: n.source_line
				});
			}
		}
		return t;
	});
	function P() {
		S.value ? F(S.value) : (C.value = [], w.value = []);
	}
	function F(e) {
		S.value = e;
		let t = j.value, n = M.value.get(t)?.get(e) ?? [];
		w.value = n.map((e) => e.outputFile);
		let r = A.value;
		C.value = n.find((e) => e.outputFile === r)?.outputLines ?? [];
	}
	function I(e, t) {
		let n = N.value.get(e)?.get(t);
		if (!n) {
			R();
			return;
		}
		S.value = n.sourceLine;
		let r = M.value.get(n.sourceFile)?.get(n.sourceLine) ?? [];
		w.value = r.map((e) => e.outputFile), C.value = r.find((t) => t.outputFile === e)?.outputLines ?? [];
	}
	function L(e, t) {
		return N.value.get(e)?.get(t)?.sourceFile;
	}
	function R() {
		S.value = null, C.value = [], w.value = [];
	}
	async function z() {
		h.value = !0, u.value = "", d.value = "", f.value = "", m.value = [];
		try {
			let e = { source: l.value };
			b.value && (e.project_dir = b.value);
			let t = await (await fetch(`${n}/run`, {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify(e)
			})).json();
			u.value = t.stdout || "", d.value = t.stderr || "", p.value = t.time_ms || 0, m.value = t.bytecode || [], t.result !== void 0 && t.result !== null && t.result !== "" && (f.value = t.result);
		} catch (e) {
			d.value = `Network error: ${e.message}`;
		} finally {
			h.value = !1;
		}
	}
	async function B(e) {
		let t = x.value[e]?.files[0]?.code ?? "";
		if (!t.trim()) {
			d.value = `No ${e} code to run. Make sure the transpilation succeeded.`;
			return;
		}
		h.value = !0, u.value = "", d.value = "", f.value = "";
		try {
			if (e === "typescript") {
				let e = await Et(t);
				u.value = e.stdout, d.value = e.stderr, p.value = 0;
			} else {
				let r = await (await fetch(`${n}/run_code`, {
					method: "POST",
					headers: { "Content-Type": "application/json" },
					body: JSON.stringify({
						language: e,
						code: t
					})
				})).json();
				u.value = r.stdout || "", d.value = r.stderr || "", p.value = r.time_ms || 0, r.result !== void 0 && r.result !== null && r.result !== "" && (f.value = r.result);
			}
		} catch (e) {
			d.value = `Network error: ${e.message}`;
		} finally {
			h.value = !1;
		}
	}
	async function V(e) {
		h.value = !0;
		try {
			let t = {
				source: l.value,
				target: e
			};
			b.value && (t.project_dir = b.value);
			let r = await (await fetch(`${n}/trans`, {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify(t)
			})).json(), i = r.files ?? [], a = {};
			for (let e of i) a[e.path] = e.source_map ?? r.source_map ?? [];
			let o = i[0]?.path ?? "";
			_.value = e, x.value[e] = {
				files: i,
				fileSourceMaps: a,
				selectedFile: o
			}, P();
		} catch (t) {
			_.value = e, x.value[e] = {
				files: [{
					path: "error.txt",
					code: `Error: ${t.message}`
				}],
				fileSourceMaps: { "error.txt": [] },
				selectedFile: "error.txt"
			}, P();
		} finally {
			h.value = !1;
		}
	}
	function H(e) {
		if (g.value = e, _.value = e, !x.value[e] && v.value) {
			V(e);
			return;
		}
		P();
	}
	function U(e, t) {
		let n = x.value[e];
		n && (n.selectedFile = t, P());
	}
	function W(e) {
		l.value = e.source, b.value = e.project_dir, u.value = "", d.value = "", f.value = "", m.value = [], S.value = null, C.value = [], w.value = [];
	}
	function ee() {
		if (typeof window > "u") return "";
		let e = JSON.stringify({
			source: l.value,
			activeTab: g.value,
			liveCompile: v.value,
			projectDir: b.value
		}), t = "#share=" + encodeURIComponent(btoa(e));
		return window.location.origin + window.location.pathname + t;
	}
	async function te() {
		let e = ee(), t = !1;
		try {
			await navigator.clipboard.writeText(e), t = !0;
		} catch {
			let n = document.createElement("textarea");
			n.value = e, document.body.appendChild(n), n.select();
			try {
				t = document.execCommand("copy");
			} catch {}
			document.body.removeChild(n);
		}
		T.value = {
			message: t ? "Share link copied to clipboard!" : "Failed to copy link",
			visible: !0
		}, setTimeout(() => {
			T.value.visible = !1;
		}, 2500);
	}
	O(l, () => {
		x.value = {}, v.value && (E && clearTimeout(E), E = setTimeout(() => {
			V(g.value);
		}, Dt));
	}), O([
		l,
		g,
		v,
		b
	], ([e, t, n, r]) => {
		s({
			source: e,
			activeTab: t,
			liveCompile: n,
			projectDir: r
		});
	}, { deep: !0 }), a && typeof window < "u" && setTimeout(() => {
		G();
	}, 100);
	async function G() {
		let e = [
			"rust",
			"c",
			"python",
			"typescript",
			"abt"
		];
		h.value = !0;
		try {
			let t = await Promise.all(e.map(async (e) => {
				try {
					let t = {
						source: l.value,
						target: e
					};
					b.value && (t.project_dir = b.value);
					let r = await (await fetch(`${n}/trans`, {
						method: "POST",
						headers: { "Content-Type": "application/json" },
						body: JSON.stringify(t)
					})).json(), i = r.files ?? [], a = {};
					for (let e of i) a[e.path] = e.source_map ?? r.source_map ?? [];
					return {
						target: e,
						files: i,
						fileSourceMaps: a,
						selectedFile: i[0]?.path ?? ""
					};
				} catch (t) {
					return {
						target: e,
						files: [{
							path: "error.txt",
							code: `Error: ${t.message}`
						}],
						fileSourceMaps: { "error.txt": [] },
						selectedFile: "error.txt"
					};
				}
			}));
			for (let e of t) x.value[e.target] = {
				files: e.files,
				fileSourceMaps: e.fileSourceMaps,
				selectedFile: e.selectedFile
			};
			let r = g.value;
			x.value[r] && (_.value = r, P());
		} finally {
			h.value = !1;
		}
	}
	let K = y(null), q = y(null), J = y([]), Y = y([]), ne = y(!1);
	async function re() {
		h.value = !0, d.value = "";
		try {
			let e = await (await fetch(`${n}/agent-debug/start`, {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify({ source: l.value })
			})).json();
			K.value = e.session_id, J.value = (e.bytecode || []).map((e) => ({
				offset: e.offset ?? e.idx ?? 0,
				mnemonic: e.mnemonic ?? e.op ?? "",
				operands: e.operands ?? e.args ?? "",
				line: e.line
			})), ne.value = !0, q.value = null, Y.value.length > 0 && await ie(Y.value);
		} catch (e) {
			d.value = `Debug start error: ${e.message}`;
		} finally {
			h.value = !1;
		}
	}
	async function ie(e) {
		if (Y.value = e, K.value) try {
			await fetch(`${n}/agent-debug/${K.value}/breakpoints`, {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify({ lines: e })
			});
		} catch {}
	}
	async function X(e) {
		if (K.value) {
			h.value = !0;
			try {
				let t = await (await fetch(`${n}/agent-debug/${K.value}/command`, {
					method: "POST",
					headers: { "Content-Type": "application/json" },
					body: JSON.stringify({ cmd: e })
				})).json();
				q.value = t, t.stdout && (u.value = t.stdout), t.stderr && (d.value = t.stderr), t.result && (f.value = t.result), (t.status === "finished" || t.status === "error") && (ne.value = !1);
			} catch (e) {
				d.value = `Debug command error: ${e.message}`;
			} finally {
				h.value = !1;
			}
		}
	}
	async function ae() {
		if (K.value) {
			try {
				await fetch(`${n}/agent-debug/${K.value}`, { method: "DELETE" });
			} catch {}
			K.value = null, q.value = null, J.value = [], ne.value = !1;
		}
	}
	return {
		source: l,
		stdout: u,
		stderr: d,
		resultCode: f,
		timeMs: p,
		runBytecode: m,
		isLoading: h,
		activeTab: g,
		transpiledCode: D,
		transpileTarget: _,
		liveCompile: v,
		projectDir: b,
		transFiles: k,
		selectedTransFile: A,
		highlightedSourceLine: S,
		highlightedOutputLines: C,
		highlightedOutputFiles: w,
		shareToast: T,
		debugSessionId: K,
		debugState: q,
		bytecode: J,
		breakpoints: Y,
		isDebugging: ne,
		run: z,
		runCode: B,
		transpile: V,
		switchTab: H,
		selectTransFile: U,
		loadExample: W,
		highlightSourceLine: F,
		highlightOutputLine: I,
		getSourceFileForOutputLine: L,
		clearHighlight: R,
		share: te,
		debugStart: re,
		debugSetBreakpoints: ie,
		debugCommand: X,
		debugStop: ae
	};
}
//#endregion
//#region src/components/SnippetRunner.vue?vue&type=script&setup=true&lang.ts
var At = {
	key: 0,
	class: "snippet-actionbar"
}, jt = ["title"], Mt = {
	key: 0,
	class: "target-hint"
}, Nt = {
	key: 1,
	class: "snippet-output"
}, Pt = {
	key: 0,
	class: "no-backend-hint"
}, Ft = /* @__PURE__ */ $(/* @__PURE__ */ l({
	__name: "SnippetRunner",
	props: {
		code: {},
		apiBase: { default: "" },
		autorun: {
			type: Boolean,
			default: !1
		},
		target: { default: "run" },
		height: { default: "auto" },
		actionBar: {
			type: Boolean,
			default: !0
		},
		fill: {
			type: Boolean,
			default: !1
		},
		orientation: { default: "stack" },
		isDebugging: {
			type: Boolean,
			default: !1
		},
		breakpoints: { default: () => [] },
		currentDebugLine: { default: null },
		runHandler: {}
	},
	emits: ["breakpoints-change"],
	setup(e, { expose: o, emit: l }) {
		let u = e, d = l, { source: f, stdout: h, stderr: _, resultCode: b, timeMs: S, isLoading: T, transpiledCode: E, liveCompile: D, transFiles: O, selectedTransFile: k, highlightedOutputLines: A, shareToast: j, debugState: M, bytecode: N, breakpoints: P, isDebugging: F, run: I, switchTab: L, selectTransFile: R, loadExample: z, share: B, highlightOutputLine: V, transpile: H, debugStart: U, debugSetBreakpoints: W, debugCommand: ee, debugStop: te } = kt({
			apiBase: u.apiBase || "/api",
			defaultSource: u.code,
			persistKey: !1,
			preloadTargets: !1
		}), G = y(!1), K = {
			rust: "Rust",
			c: "C",
			python: "Python",
			typescript: "TypeScript",
			abt: "ABT"
		}, q = t(() => u.target === "run" ? "" : K[u.target]), J = t(() => u.target === "run" ? "Run (Ctrl+Enter)" : `Transpile to ${q.value}`), Y = t(() => _.value.startsWith("Network error")), ne = t(() => {
			if (u.fill || u.height !== "auto") return {};
			let e = f.value.split("\n").length;
			return { height: `${Math.min(480, Math.max(80, e * 20 + 16))}px` };
		}), re = t(() => u.fill || u.height === "auto" ? {} : { height: u.height });
		async function ie() {
			u.target === "run" ? await I() : await H(u.target), (h.value || _.value || b.value || E.value) && (G.value = !0);
		}
		function X() {
			if (u.runHandler) {
				u.runHandler();
				return;
			}
			ie();
		}
		return u.autorun && g(() => {
			X();
		}), o({
			source: f,
			stdout: h,
			stderr: _,
			resultCode: b,
			timeMs: S,
			isLoading: T,
			transpiledCode: E,
			liveCompile: D,
			transFiles: O,
			selectedTransFile: k,
			highlightedOutputLines: A,
			shareToast: j,
			debugState: M,
			bytecode: N,
			breakpoints: P,
			isDebugging: F,
			run: I,
			switchTab: L,
			selectTransFile: R,
			loadExample: z,
			share: B,
			highlightOutputLine: V,
			transpile: H,
			debugStart: U,
			debugSetBreakpoints: W,
			debugCommand: ee,
			debugStop: te,
			action: X
		}), (t, o) => (v(), i("div", {
			class: p(["snippet-runner", {
				fill: e.fill,
				row: e.fill && e.orientation === "row",
				sized: !e.fill && e.height !== "auto"
			}]),
			style: m(re.value)
		}, [
			e.actionBar ? (v(), i("div", At, [
				a("button", {
					class: p(["run-action", { busy: w(T) }]),
					title: J.value,
					onClick: X
				}, [w(T) ? (v(), n(w(Se), {
					key: 1,
					size: 13,
					class: "spin"
				})) : (v(), n(w(Ce), {
					key: 0,
					size: 13
				}))], 10, jt),
				e.target === "run" ? r("", !0) : (v(), i("span", Mt, "→ " + C(q.value), 1)),
				o[3] ||= a("span", { class: "spacer" }, null, -1),
				a("button", {
					class: p(["output-toggle", { open: G.value }]),
					title: "Toggle output",
					onClick: o[0] ||= (e) => G.value = !G.value
				}, [c(w(pe), { size: 13 })], 2)
			])) : r("", !0),
			a("div", {
				class: "snippet-editor",
				style: m(ne.value)
			}, [c(Je, {
				"model-value": w(f),
				"onUpdate:modelValue": o[1] ||= (e) => f.value = e,
				"on-run": X,
				"is-debugging": w(F),
				breakpoints: w(P),
				"current-debug-line": e.currentDebugLine ?? null,
				onBreakpointsChange: o[2] ||= (e) => d("breakpoints-change", e)
			}, null, 8, [
				"model-value",
				"is-debugging",
				"breakpoints",
				"current-debug-line"
			])], 4),
			G.value || e.fill ? (v(), i("div", Nt, [x(t.$slots, "output", {}, () => [Y.value ? (v(), i("div", Pt, [...o[4] ||= [s(" 后端不可用——本地启动：", -1), a("code", null, "cargo run -p auto-playground", -1)]])) : r("", !0), e.target === "run" ? (v(), n(St, {
				key: 1,
				stdout: w(h),
				stderr: w(_),
				result: w(b),
				"time-ms": w(S)
			}, null, 8, [
				"stdout",
				"stderr",
				"result",
				"time-ms"
			])) : (v(), n(gt, {
				key: 2,
				code: w(E),
				language: e.target
			}, null, 8, ["code", "language"]))], !0)])) : r("", !0)
		], 6));
	}
}), [["__scopeId", "data-v-af98db41"]]), It = ["data-offset", "onClick"], Lt = { class: "offset" }, Rt = { class: "mnemonic" }, zt = { class: "operands" }, Bt = ["data-tip"], Vt = 320, Ht = /* @__PURE__ */ $(/* @__PURE__ */ l({
	__name: "BytecodePanel",
	props: {
		bytecode: {},
		bytecodeMeta: {},
		currentIp: {},
		selectedOffsets: {},
		highlightedOffsets: {}
	},
	emits: ["offsetClick"],
	setup(t) {
		let o = t;
		function c(e) {
			let t = e.replace(/\\/g, "\\\\").replace(/\n/g, "⏎").replace(/\r/g, "").replace(/\t/g, " ");
			return t.length > Vt ? t.slice(0, Vt) + "…" : t;
		}
		function l(e) {
			let t = o.bytecodeMeta?.strings[e];
			return t === void 0 ? void 0 : `"${c(t)}"`;
		}
		function u(e) {
			let t = o.bytecodeMeta?.natives[String(e)];
			return t === void 0 ? void 0 : c(t);
		}
		function d(e) {
			let t = o.bytecodeMeta?.functions.find((t) => t.offset === e);
			return t ? `fn ${t.name}` : void 0;
		}
		function h(e) {
			let t;
			if (t = e.match(/^str\[(\d+)\]$/)) return l(Number(t[1]));
			if (t = e.match(/^field\[(\d+)\]$/)) return o.bytecodeMeta?.strings[Number(t[1])];
			if (t = e.match(/^nat#(\d+)$/)) return u(Number(t[1]));
			if (t = e.match(/^method[=-](\d+)(,.*)?$/)) return l(Number(t[1]));
			if (t = e.match(/^(?:-> )?(?:addr=)?(0x[0-9a-fA-F]+)$/)) return d(parseInt(t[1], 16));
		}
		let g = /(str\[\d+\]|field\[\d+\]|nat#\d+|(?:-> )?addr=0x[0-9a-fA-F]+|-> 0x[0-9a-fA-F]+|\b0x[0-9a-fA-F]+\b|method[=-]\d+)/g;
		function _(e) {
			let t = e.operands;
			if (!t) return [{
				text: "",
				tip: void 0
			}];
			if (/^(load\.global|store\.global)$/.test(e.mnemonic) && /^\d+$/.test(t.trim()) && o.bytecodeMeta) {
				let e = Number(t.trim()), n = o.bytecodeMeta.strings[e];
				return [{
					text: t,
					tip: n === void 0 ? void 0 : String(e) + ": \"" + c(n) + "\""
				}];
			}
			let n = [], r = 0;
			for (let e of t.matchAll(g)) {
				let i = e.index ?? 0;
				i > r && n.push({ text: t.slice(r, i) });
				let a = e[0];
				n.push({
					text: a,
					tip: h(a)
				}), r = i + a.length;
			}
			return r < t.length && n.push({ text: t.slice(r) }), n;
		}
		let x = y(null), S = y({
			visible: !1,
			text: "",
			x: 0,
			y: 0
		});
		function w() {
			S.value.visible = !1;
		}
		function T(e) {
			let t = e.target.closest?.(".tok"), n = x.value?.$el ?? null;
			if (!t || !n || !t.dataset.tip) {
				w();
				return;
			}
			let r = n.getBoundingClientRect(), i = t.getBoundingClientRect(), a = Math.min(r.width - 16, 480);
			S.value = {
				visible: !0,
				text: t.dataset.tip,
				x: Math.max(6, Math.min(i.left - r.left, r.width - a - 10)),
				y: i.bottom - r.top + 4
			};
		}
		function E(e) {
			return e.toString(16).padStart(4, "0");
		}
		return O(() => o.selectedOffsets, async (e) => {
			e?.length && (await f(), x.value?.$el?.querySelector(`[data-offset="${e[0]}"]`)?.scrollIntoView({ block: "nearest" }));
		}), (o, c) => (v(), n(Ye, {
			ref_key: "panelRef",
			ref: x,
			class: "bytecode-panel",
			onMouseover: T,
			onMouseleave: w
		}, {
			default: k(() => [(v(!0), i(e, null, b(t.bytecode, (n) => (v(), i("div", {
				key: n.offset,
				"data-offset": n.offset,
				class: p(["bytecode-line", {
					"is-current": n.offset === t.currentIp,
					"is-selected": t.selectedOffsets?.includes(n.offset),
					"is-hover": t.highlightedOffsets?.includes(n.offset),
					"has-source": n.line !== void 0
				}]),
				onClick: (e) => o.$emit("offsetClick", n.offset)
			}, [
				a("span", Lt, C(E(n.offset)), 1),
				a("span", Rt, C(n.mnemonic), 1),
				a("span", zt, [(v(!0), i(e, null, b(_(n), (t, n) => (v(), i(e, { key: n }, [t.tip ? (v(), i("span", {
					key: 0,
					class: "tok",
					"data-tip": t.tip
				}, C(t.text), 9, Bt)) : (v(), i(e, { key: 1 }, [s(C(t.text), 1)], 64))], 64))), 128))])
			], 10, It))), 128)), S.value.visible ? (v(), i("div", {
				key: 0,
				class: "tok-tooltip",
				style: m({
					left: S.value.x + "px",
					top: S.value.y + "px"
				})
			}, C(S.value.text), 5)) : r("", !0)]),
			_: 1
		}, 512));
	}
}), [["__scopeId", "data-v-96c74ca0"]]), Ut = { label: "Single-file" }, Wt = ["value"], Gt = { label: "Projects" }, Kt = ["value"], qt = /* @__PURE__ */ $(/* @__PURE__ */ l({
	__name: "ExampleSelector",
	props: { apiBase: { default: "/api" } },
	emits: ["select"],
	setup(n, { emit: r }) {
		let o = n, s = r, c = y([]), l = y(""), u = t(() => c.value.filter((e) => e.example_type === "single")), d = t(() => c.value.filter((e) => e.example_type === "project"));
		g(async () => {
			try {
				c.value = (await (await fetch(`${o.apiBase}/examples`)).json()).examples || [];
			} catch {}
		});
		function f() {
			if (l.value) {
				try {
					let e = JSON.parse(l.value);
					s("select", {
						source: e.source,
						project_dir: e.project_dir,
						files: e.files
					});
				} catch {}
				l.value = "";
			}
		}
		return (t, n) => A((v(), i("select", {
			class: "example-selector",
			onChange: f,
			"onUpdate:modelValue": n[0] ||= (e) => l.value = e
		}, [
			n[1] ||= a("option", { value: "" }, "Load Example...", -1),
			a("optgroup", Ut, [(v(!0), i(e, null, b(u.value, (e) => (v(), i("option", {
				key: e.name,
				value: JSON.stringify(e)
			}, C(e.name), 9, Wt))), 128))]),
			a("optgroup", Gt, [(v(!0), i(e, null, b(d.value, (e) => (v(), i("option", {
				key: e.name,
				value: JSON.stringify(e)
			}, C(e.name), 9, Kt))), 128))])
		], 544)), [[E, l.value]]);
	}
}), [["__scopeId", "data-v-578bba03"]]), Jt = ["onClick"], Yt = { class: "file-name" }, Xt = /* @__PURE__ */ $(/* @__PURE__ */ l({
	__name: "FileTree",
	props: {
		files: {},
		selected: {},
		mappedFiles: {}
	},
	emits: ["select"],
	setup(t) {
		return (r, o) => (v(), n(Ye, { class: "file-tree" }, {
			default: k(() => [(v(!0), i(e, null, b(t.files, (e) => (v(), i("div", {
				key: e.path,
				class: p(["file-item", {
					active: e.path === t.selected,
					mapped: t.mappedFiles?.includes(e.path)
				}]),
				onClick: (t) => r.$emit("select", e.path)
			}, [a("span", Yt, C(e.path), 1)], 10, Jt))), 128))]),
			_: 1
		}));
	}
}), [["__scopeId", "data-v-2da68091"]]), Zt = { class: "card-toolbar" }, Qt = { class: "toolbar-left" }, $t = { class: "toolbar-right" }, en = ["disabled"], tn = ["disabled"], nn = {
	key: 2,
	class: "debug-controls"
}, rn = ["disabled"], an = ["disabled"], on = ["disabled"], sn = ["disabled"], cn = ["disabled"], ln = {
	key: 5,
	class: "switch-widget",
	title: "Toggle live transpile on edit"
}, un = { class: "switch" }, dn = ["checked"], fn = { class: "card-body" }, pn = { class: "card-output" }, mn = { class: "output-tabs" }, hn = ["onClick"], gn = ["title"], _n = { class: "output-content" }, vn = {
	key: 2,
	class: "output-code-split"
}, yn = {
	key: 0,
	class: "debug-panel"
}, bn = {
	key: 0,
	class: "debug-section"
}, xn = { class: "debug-section-title" }, Sn = { class: "debug-stack" }, Cn = {
	key: 1,
	class: "debug-section"
}, wn = { class: "frame-name" }, Tn = { class: "frame-info" }, En = {
	key: 2,
	class: "debug-section"
}, Dn = { class: "debug-locals" }, On = { class: "local-idx" }, kn = {
	key: 3,
	class: "debug-registers"
}, An = "fn main() {\n    let message = \"Hello from Auto!\"\n    print(message)\n}", jn = /* @__PURE__ */ $(/* @__PURE__ */ l({
	__name: "PlaygroundCard",
	props: {
		code: {},
		apiBase: { default: "" },
		autorun: {
			type: Boolean,
			default: !1
		},
		target: { default: "run" },
		height: { default: "480px" },
		noteId: {},
		toolbar: { default: () => ({}) },
		exampleSelector: {
			type: Boolean,
			default: !1
		}
	},
	setup(l, { expose: u }) {
		let d = l, f = t(() => d.code ?? An), h = t(() => ({
			transpile: d.toolbar?.transpile ?? !0,
			share: d.toolbar?.share ?? !0,
			debug: d.toolbar?.debug ?? !0,
			live: d.toolbar?.live ?? !0
		})), g = y(null), _ = t(() => g.value?.isLoading ?? !1), x = t(() => g.value?.isDebugging ?? !1), S = t(() => g.value?.debugState ?? null), T = t(() => g.value?.stdout ?? ""), D = t(() => g.value?.stderr ?? ""), j = t(() => g.value?.resultCode ?? ""), M = t(() => g.value?.timeMs ?? 0), N = t(() => g.value?.bytecode ?? []), P = t(() => g.value?.transpiledCode ?? ""), F = t(() => g.value?.transFiles ?? []), I = t(() => g.value?.selectedTransFile ?? ""), L = t(() => g.value?.highlightedOutputLines ?? []), R = t(() => g.value?.liveCompile ?? !1), z = t(() => g.value?.breakpoints ?? []), B = t(() => g.value?.shareToast), V = y("Output"), H = y(d.target), U = y(!1), W = [
			"Output",
			"rust",
			"c",
			"python",
			"typescript",
			"abt",
			"Bytecode"
		], ee = {
			Output: "Output",
			rust: "Rust",
			c: "C",
			python: "Python",
			typescript: "TS",
			abt: "ABT",
			Bytecode: "Bytecode"
		}, te = t(() => d.height === "auto" ? { minHeight: "480px" } : { height: d.height }), G = t(() => V.value !== "Output" && V.value !== "Bytecode" && !!P.value), K = t(() => {
			let e = V.value;
			return e !== "Output" && e !== "Bytecode" && F.value.length > 1;
		});
		async function q() {
			let e = g.value;
			e && (H.value === "run" ? (await e.run(), V.value = "Output") : (e.switchTab(H.value), V.value = H.value));
		}
		O(H, (e) => {
			e !== "run" && R.value && (g.value?.switchTab(e), V.value = e);
		});
		function J(e) {
			V.value = e, e !== "Output" && e !== "Bytecode" ? (H.value = e, g.value?.switchTab(e)) : e === "Output" && (H.value = "run");
		}
		function Y(e) {
			g.value?.selectTransFile(V.value, e);
		}
		function ne(e) {
			g.value?.loadExample(e), V.value = "Output", H.value = "run";
		}
		function re(e) {
			g.value?.highlightOutputLine(I.value, e);
		}
		function ie(e) {
			let t = g.value;
			t && (t.liveCompile = e.target.checked);
		}
		async function X() {
			await g.value?.share();
		}
		async function ae() {
			if (P.value) try {
				await navigator.clipboard.writeText(P.value), U.value = !0, setTimeout(() => {
					U.value = !1;
				}, 2e3);
			} catch {}
		}
		async function oe() {
			await g.value?.debugStart(), V.value = "Bytecode";
		}
		async function se() {
			await g.value?.debugStop(), V.value = "Output";
		}
		function Z(e) {
			g.value?.debugCommand(e);
		}
		function ce(e) {
			g.value?.debugSetBreakpoints(e);
		}
		function Q(e) {}
		return u({ run: q }), (t, u) => (v(), i(e, null, [a("div", {
			class: "playground-card",
			style: m(te.value)
		}, [a("div", Zt, [a("div", Qt, [
			c(w(ve), { size: 16 }),
			u[6] ||= a("span", { class: "toolbar-title" }, "Auto Playground", -1),
			l.exampleSelector ? (v(), n(qt, {
				key: 0,
				"api-base": l.apiBase || "/api",
				onSelect: ne
			}, null, 8, ["api-base"])) : r("", !0)
		]), a("div", $t, [
			h.value.transpile ? A((v(), i("select", {
				key: 0,
				"onUpdate:modelValue": u[0] ||= (e) => H.value = e,
				class: "target-select",
				disabled: !!x.value
			}, [...u[7] ||= [o("<option value=\"run\" data-v-e009b362>Run</option><option value=\"rust\" data-v-e009b362>→ Rust</option><option value=\"c\" data-v-e009b362>→ C</option><option value=\"python\" data-v-e009b362>→ Python</option><option value=\"typescript\" data-v-e009b362>→ TypeScript</option><option value=\"abt\" data-v-e009b362>→ ABT</option>", 6)]], 8, en)), [[E, H.value]]) : r("", !0),
			x.value ? h.value.debug ? (v(), i("div", nn, [
				a("button", {
					class: "debug-btn continue",
					onClick: u[1] ||= (e) => Z("continue"),
					disabled: _.value,
					title: "Continue"
				}, [c(w(Ce), { size: 14 })], 8, rn),
				a("button", {
					class: "debug-btn step",
					onClick: u[2] ||= (e) => Z("step"),
					disabled: _.value,
					title: "Step Into"
				}, [c(w(le), { size: 14 })], 8, an),
				a("button", {
					class: "debug-btn step-over",
					onClick: u[3] ||= (e) => Z("step_over"),
					disabled: _.value,
					title: "Step Over"
				}, [c(w(De), { size: 14 })], 8, on),
				a("button", {
					class: "debug-btn step-out",
					onClick: u[4] ||= (e) => Z("step_out"),
					disabled: _.value,
					title: "Step Out"
				}, [c(w(ue), { size: 14 })], 8, sn)
			])) : r("", !0) : (v(), i("button", {
				key: 1,
				class: "run-btn",
				onClick: q,
				disabled: _.value
			}, [_.value ? (v(), n(w(Se), {
				key: 1,
				size: 14,
				class: "spin"
			})) : (v(), n(w(Ce), {
				key: 0,
				size: 14
			})), s(" " + C(_.value ? "Running..." : "Run"), 1)], 8, tn)),
			h.value.debug && x.value ? (v(), i("button", {
				key: 3,
				class: "stop-btn",
				onClick: se,
				title: "Stop Debug"
			}, [c(w(Oe), { size: 14 }), u[8] ||= s(" Stop ", -1)])) : h.value.debug ? (v(), i("button", {
				key: 4,
				class: "debug-start-btn",
				onClick: oe,
				disabled: _.value,
				title: "Start Debug"
			}, [c(w(de), { size: 14 }), u[9] ||= s(" Debug ", -1)], 8, cn)) : r("", !0),
			h.value.live && !x.value ? (v(), i("label", ln, [u[11] ||= a("span", { class: "switch-label" }, "Live", -1), a("span", un, [a("input", {
				type: "checkbox",
				checked: !!R.value,
				onChange: u[5] ||= (e) => ie(e)
			}, null, 40, dn), u[10] ||= a("span", { class: "slider" }, null, -1)])])) : r("", !0),
			h.value.share ? (v(), i("button", {
				key: 6,
				class: "icon-btn share-btn",
				onClick: X,
				title: "Copy shareable link"
			}, [c(w(Ee), { size: 14 })])) : r("", !0)
		])]), a("div", fn, [c(Ft, {
			ref_key: "runner",
			ref: g,
			code: f.value,
			"api-base": l.apiBase,
			autorun: l.autorun,
			height: l.height,
			"action-bar": !1,
			fill: "",
			orientation: "row",
			"is-debugging": !!x.value,
			breakpoints: z.value ?? [],
			"current-debug-line": S.value?.line ?? null,
			"run-handler": q,
			onBreakpointsChange: ce
		}, {
			output: k(() => [a("div", pn, [
				a("div", mn, [
					(v(), i(e, null, b(W, (e) => a("button", {
						key: e,
						class: p(["tab-btn", { active: V.value === e }]),
						onClick: (t) => J(e)
					}, C(ee[e]), 11, hn)), 64)),
					u[12] ||= a("div", { class: "spacer" }, null, -1),
					x.value && S.value ? (v(), i("span", {
						key: 0,
						class: p(["debug-status", S.value.status])
					}, C(S.value.status), 3)) : r("", !0),
					G.value ? (v(), i("button", {
						key: 1,
						class: "icon-btn copy-btn",
						onClick: ae,
						title: U.value ? "Copied!" : "Copy code"
					}, [U.value ? (v(), n(w(fe), {
						key: 1,
						size: 14
					})) : (v(), n(w(ye), {
						key: 0,
						size: 14
					}))], 8, gn)) : r("", !0)
				]),
				a("div", _n, [V.value === "Output" ? (v(), n(St, {
					key: 0,
					stdout: T.value ?? "",
					stderr: D.value ?? "",
					result: j.value ?? "",
					"time-ms": M.value ?? 0
				}, null, 8, [
					"stdout",
					"stderr",
					"result",
					"time-ms"
				])) : V.value === "Bytecode" ? (v(), n(Ht, {
					key: 1,
					bytecode: N.value ?? [],
					"current-ip": S.value?.ip,
					onOffsetClick: Q
				}, null, 8, ["bytecode", "current-ip"])) : (v(), i("div", vn, [K.value ? (v(), n(Xt, {
					key: 0,
					files: F.value ?? [],
					selected: I.value ?? "",
					onSelect: Y
				}, null, 8, ["files", "selected"])) : r("", !0), c(gt, {
					code: P.value ?? "",
					language: V.value,
					"highlight-lines": L.value ?? [],
					onLineClick: re
				}, null, 8, [
					"code",
					"language",
					"highlight-lines"
				])]))]),
				x.value && S.value ? (v(), i("div", yn, [
					S.value.stack.length ? (v(), i("div", bn, [a("div", xn, "Stack (" + C(S.value.stack.length) + ")", 1), a("div", Sn, [(v(!0), i(e, null, b(S.value.stack.slice(-8), (e, t) => (v(), i("span", {
						key: t,
						class: "stack-item"
					}, C(e), 1))), 128))])])) : r("", !0),
					S.value.call_stack.length ? (v(), i("div", Cn, [u[13] ||= a("div", { class: "debug-section-title" }, "Call Stack", -1), (v(!0), i(e, null, b(S.value.call_stack, (e, t) => (v(), i("div", {
						key: t,
						class: "call-frame"
					}, [a("span", wn, C(e.fn_name || "<root>"), 1), a("span", Tn, "line " + C(e.line) + ", bp=" + C(e.bp), 1)]))), 128))])) : r("", !0),
					S.value.locals.length ? (v(), i("div", En, [u[14] ||= a("div", { class: "debug-section-title" }, "Locals", -1), a("div", Dn, [(v(!0), i(e, null, b(S.value.locals, (e, t) => (v(), i("span", {
						key: t,
						class: "local-item"
					}, [a("span", On, "[" + C(e.index) + "]", 1), s(" " + C(e.value), 1)]))), 128))])])) : r("", !0),
					S.value.registers ? (v(), i("div", kn, " IP=" + C(S.value.registers.ip) + " BP=" + C(S.value.registers.bp) + " SP=" + C(S.value.registers.sp), 1)) : r("", !0)
				])) : r("", !0)
			])]),
			_: 1
		}, 8, [
			"code",
			"api-base",
			"autorun",
			"height",
			"is-debugging",
			"breakpoints",
			"current-debug-line"
		])])], 4), a("div", { class: p(["toast", { visible: B.value?.visible }]) }, C(B.value?.message), 3)], 64));
	}
}), [["__scopeId", "data-v-e009b362"]]), Mn = /* @__PURE__ */ l({
	__name: "AutoPlayground",
	props: {
		code: { default: "fn main() {\n    let message = \"Hello from Auto!\"\n    print(message)\n}" },
		apiUrl: { default: "" },
		height: { default: "500px" }
	},
	setup(e) {
		let t = e, r = t.apiUrl ? `${t.apiUrl}/api` : "";
		return (t, i) => (v(), n(jn, {
			code: e.code,
			"api-base": w(r),
			height: e.height
		}, null, 8, [
			"code",
			"api-base",
			"height"
		]));
	}
}), Nn = { class: "debug-toolbar" }, Pn = [
	"disabled",
	"onClick",
	"title"
], Fn = { class: "icon" }, In = { class: "label" }, Ln = ["title"], Rn = { class: "icon" }, zn = { class: "label" }, Bn = ["disabled"], Vn = /* @__PURE__ */ $(/* @__PURE__ */ l({
	__name: "DebugToolbar",
	props: {
		isPaused: { type: Boolean },
		isRecording: { type: Boolean },
		hasRecording: { type: Boolean }
	},
	emits: [
		"command",
		"toggleRecord",
		"exportRecording"
	],
	setup(t) {
		let n = [
			{
				cmd: "continue",
				icon: "▶",
				label: "Continue",
				title: "Continue (F5)"
			},
			{
				cmd: "step",
				icon: "↓",
				label: "Step Into",
				title: "Step Into (F11)"
			},
			{
				cmd: "step_over",
				icon: "→",
				label: "Step Over",
				title: "Step Over (F10)"
			},
			{
				cmd: "step_out",
				icon: "↑",
				label: "Step Out",
				title: "Step Out (Shift+F11)"
			}
		];
		return (r, o) => (v(), i("div", Nn, [
			(v(), i(e, null, b(n, (e) => a("button", {
				key: e.cmd,
				disabled: !t.isPaused,
				onClick: (t) => r.$emit("command", e.cmd),
				title: e.title
			}, [a("span", Fn, C(e.icon), 1), a("span", In, C(e.label), 1)], 8, Pn)), 64)),
			a("button", {
				class: "stop-btn",
				onClick: o[0] ||= (e) => r.$emit("command", "stop"),
				title: "Stop Debugging (Shift+F5)"
			}, [...o[3] ||= [a("span", { class: "icon" }, "■", -1), a("span", { class: "label" }, "Stop", -1)]]),
			o[5] ||= a("div", { class: "toolbar-divider" }, null, -1),
			a("button", {
				class: p(["record-btn", { recording: t.isRecording }]),
				onClick: o[1] ||= (e) => r.$emit("toggleRecord"),
				title: t.isRecording ? "Stop Recording" : "Start Recording"
			}, [a("span", Rn, C(t.isRecording ? "⏹" : "⏺"), 1), a("span", zn, C(t.isRecording ? "Recording" : "Record"), 1)], 10, Ln),
			a("button", {
				class: "save-btn",
				onClick: o[2] ||= (e) => r.$emit("exportRecording"),
				disabled: !t.hasRecording,
				title: "Export Replay File"
			}, [...o[4] ||= [a("span", { class: "icon" }, "💾", -1), a("span", { class: "label" }, "Save", -1)]], 8, Bn)
		]));
	}
}), [["__scopeId", "data-v-8068f5da"]]), Hn = { class: "replay-toolbar" }, Un = ["title"], Wn = { class: "icon" }, Gn = { class: "timeline" }, Kn = ["max", "value"], qn = { class: "frame-info" }, Jn = /* @__PURE__ */ $(/* @__PURE__ */ l({
	__name: "ReplayToolbar",
	props: {
		isPlaying: { type: Boolean },
		currentIndex: {},
		totalFrames: {}
	},
	emits: [
		"play",
		"pause",
		"stepForward",
		"stepBackward",
		"seek"
	],
	setup(e, { emit: n }) {
		let r = e, o = t(() => r.currentIndex ?? 0), s = t(() => r.totalFrames ?? 0), c = n;
		function l(e) {
			c("seek", parseInt(e.target.value, 10));
		}
		return (t, n) => (v(), i("div", Hn, [
			a("button", {
				onClick: n[0] ||= (n) => e.isPlaying ? t.$emit("pause") : t.$emit("play"),
				title: e.isPlaying ? "Pause" : "Play"
			}, [a("span", Wn, C(e.isPlaying ? "⏸" : "▶"), 1)], 8, Un),
			a("button", {
				onClick: n[1] ||= (e) => t.$emit("stepBackward"),
				title: "Step Backward (←)"
			}, [...n[3] ||= [a("span", { class: "icon" }, "⏮", -1)]]),
			a("button", {
				onClick: n[2] ||= (e) => t.$emit("stepForward"),
				title: "Step Forward (→)"
			}, [...n[4] ||= [a("span", { class: "icon" }, "⏭", -1)]]),
			a("div", Gn, [a("input", {
				type: "range",
				min: 0,
				max: Math.max(0, s.value - 1),
				value: o.value,
				onInput: l,
				class: "timeline-slider"
			}, null, 40, Kn), a("span", qn, "Frame " + C(o.value + 1) + " / " + C(s.value), 1)]),
			n[5] ||= a("div", { class: "replay-badge" }, "🔁 Replay Mode", -1)
		]));
	}
}), [["__scopeId", "data-v-1b1084c5"]]), Yn = { class: "aux-section" }, Xn = {
	key: 0,
	class: "var-group"
}, Zn = { class: "var-name" }, Qn = { class: "var-value" }, $n = {
	key: 1,
	class: "var-group"
}, er = { class: "var-name" }, tr = { class: "var-value" }, nr = {
	key: 2,
	class: "var-group"
}, rr = { class: "var-name" }, ir = { class: "var-value" }, ar = {
	key: 3,
	class: "empty"
}, or = { class: "aux-section" }, sr = { class: "callstack-list" }, cr = { class: "cs-name" }, lr = { class: "cs-line" }, ur = {
	key: 0,
	class: "empty"
}, dr = { class: "aux-section compact" }, fr = { class: "reg-row" }, pr = { class: "reg-value" }, mr = { class: "reg-row" }, hr = { class: "reg-value" }, gr = { class: "reg-row" }, _r = { class: "reg-value" }, vr = {
	key: 0,
	class: "aux-section stdout-section"
}, yr = { class: "stdout-text" }, br = /* @__PURE__ */ $(/* @__PURE__ */ l({
	__name: "DebugAuxPanel",
	props: { state: {} },
	setup(o) {
		let s = o, l = y(!1), u = y(!1);
		O(() => s.state?.stack, (e, t) => {
			let n = e || [], r = t || [];
			n.length > r.length ? l.value = !0 : n.length < r.length && (u.value = !0), setTimeout(() => {
				l.value = !1, u.value = !1;
			}, 600);
		}, {
			deep: !0,
			flush: "post"
		});
		let d = t(() => {
			if (!s.state) return [];
			let e = s.state.stack, t = Math.min(e.length, 8);
			return [...e.slice(-t)].reverse().map((e, t) => ({
				value: e,
				distFromTop: t
			}));
		}), f = t(() => {
			if (!s.state) return [];
			let e = [...s.state.call_stack];
			return e.push({
				fn_name: null,
				line: s.state.line,
				return_ip: s.state.registers.ip,
				bp: s.state.registers.bp,
				n_args: s.state.args?.length ?? 0,
				n_locals: s.state.locals?.length ?? 0
			}), e.reverse();
		}), m = t(() => (s.state?.args?.length ?? 0) > 0 || (s.state?.locals?.length ?? 0) > 0 || d.value.length > 0);
		function h(e) {
			return e === void 0 ? "-" : `0x${e.toString(16).padStart(4, "0")}`;
		}
		return (t, g) => (v(), n(Ye, { class: "debug-aux-panel" }, {
			default: k(() => [
				a("div", Yn, [
					g[3] ||= a("div", { class: "aux-title" }, "Variables", -1),
					s.state?.args?.length ? (v(), i("div", Xn, [g[0] ||= a("div", { class: "var-group-title" }, "Arguments", -1), (v(!0), i(e, null, b(s.state.args, (e) => (v(), i("div", {
						key: "arg" + e.index,
						class: "var-row"
					}, [a("span", Zn, "arg" + C(e.index), 1), a("span", Qn, C(e.value), 1)]))), 128))])) : r("", !0),
					s.state?.locals?.length ? (v(), i("div", $n, [g[1] ||= a("div", { class: "var-group-title" }, "Locals", -1), (v(!0), i(e, null, b(s.state.locals, (e) => (v(), i("div", {
						key: "loc" + e.index,
						class: "var-row"
					}, [a("span", er, "local" + C(e.index), 1), a("span", tr, C(e.value), 1)]))), 128))])) : r("", !0),
					d.value.length ? (v(), i("div", nr, [g[2] ||= a("div", { class: "var-group-title" }, "Stack Top", -1), (v(!0), i(e, null, b(d.value, (e, t) => (v(), i("div", {
						key: "stk" + t,
						class: p(["var-row", {
							"is-top": t === 0,
							"is-pushed": l.value && t === 0,
							"is-popped": u.value && t === 0
						}])
					}, [a("span", rr, "[" + C(e.distFromTop) + "]", 1), a("span", ir, C(e.value), 1)], 2))), 128))])) : r("", !0),
					m.value ? r("", !0) : (v(), i("div", ar, "No variables"))
				]),
				a("div", or, [
					g[4] ||= a("div", { class: "aux-title" }, "Call Stack", -1),
					a("div", sr, [(v(!0), i(e, null, b(f.value, (e, t) => (v(), i("div", {
						key: t,
						class: p(["callstack-item", { "is-current": t === 0 }])
					}, [a("span", cr, C(e.fn_name ?? "<main>"), 1), a("span", lr, ":" + C(e.line), 1)], 2))), 128))]),
					s.state?.call_stack?.length ? r("", !0) : (v(), i("div", ur, "No frames"))
				]),
				a("div", dr, [
					g[8] ||= a("div", { class: "aux-title" }, "Registers", -1),
					a("div", fr, [g[5] ||= a("span", { class: "reg-label" }, "IP", -1), a("span", pr, C(h(o.state?.registers.ip)), 1)]),
					a("div", mr, [g[6] ||= a("span", { class: "reg-label" }, "BP", -1), a("span", hr, C(h(o.state?.registers.bp)), 1)]),
					a("div", gr, [g[7] ||= a("span", { class: "reg-label" }, "SP", -1), a("span", _r, C(h(o.state?.registers.sp)), 1)])
				]),
				o.state?.stdout ? (v(), i("div", vr, [g[9] ||= a("div", { class: "aux-title" }, "Output", -1), c(Ye, { class: "stdout-scroll" }, {
					default: k(() => [a("pre", yr, C(o.state.stdout), 1)]),
					_: 1
				})])) : r("", !0)
			]),
			_: 1
		}));
	}
}), [["__scopeId", "data-v-ee14fb49"]]), xr = { class: "playground" }, Sr = { class: "toolbar" }, Cr = { class: "toolbar-left" }, wr = { class: "toolbar-right" }, Tr = ["disabled", "title"], Er = ["disabled"], Dr = ["disabled"], Or = ["disabled"], kr = { class: "trans-current" }, Ar = { class: "workspace" }, jr = { class: "main-row" }, Mr = { class: "pane-header" }, Nr = { key: 0 }, Pr = { key: 1 }, Fr = {
	key: 0,
	class: "active-file-name"
}, Ir = { class: "pane-body" }, Lr = {
	key: 0,
	class: "preview-pane"
}, Rr = { class: "pane-header" }, zr = ["disabled"], Br = {
	key: 0,
	class: "output-pane"
}, Vr = { class: "pane-header" }, Hr = { class: "output-body" }, Ur = /* @__PURE__ */ $(/* @__PURE__ */ l({
	__name: "PlaygroundLayout",
	props: {
		source: {},
		isLoading: { type: Boolean },
		mode: {},
		transTarget: {},
		stdout: {},
		stderr: {},
		resultCode: {},
		timeMs: {},
		transpiledCode: {},
		transFiles: {},
		selectedTransFile: {},
		highlightLines: {},
		projectFiles: {},
		activeFile: {},
		mappedSourceFiles: {},
		onRun: { type: Function },
		onTrans: { type: Function },
		onRunCode: { type: Function },
		onDebug: { type: Function },
		onSelectTransFile: { type: Function },
		onOutputLineClick: { type: Function },
		isDebugging: { type: Boolean },
		isPaused: { type: Boolean },
		isRecording: { type: Boolean },
		hasRecording: { type: Boolean },
		bytecode: {},
		bytecodeMeta: {},
		selectedOffsets: {},
		selectedSourceLine: {},
		debugState: {},
		currentSourceLine: {},
		highlightedOffsets: {},
		breakpoints: {},
		currentDebugLine: {},
		isReplayMode: { type: Boolean },
		replayCurrentIndex: {},
		replayTotalFrames: {},
		isReplayPlaying: { type: Boolean },
		onHighlightLine: { type: Function },
		onClearHighlight: { type: Function }
	},
	emits: [
		"update:source",
		"update:transTarget",
		"loadExample",
		"selectFile",
		"share",
		"debugCommand",
		"toggleRecord",
		"exportRecording",
		"lineClick",
		"offsetClick",
		"breakpointsChange",
		"loadReplay",
		"replayPlay",
		"replayPause",
		"replayStepForward",
		"replayStepBackward",
		"replaySeek"
	],
	setup(l, { emit: u }) {
		let d = l, f = u, h = t({
			get: () => d.transTarget,
			set: (e) => f("update:transTarget", e)
		}), _ = t(() => ({
			rust: "Rust",
			c: "C",
			python: "Python",
			typescript: "TypeScript",
			abt: "ABT",
			bytecode: "Bytecode"
		})[d.transTarget] ?? d.transTarget), b = y(null), x = y("auto");
		g(async () => {
			await document.fonts?.ready;
			let e = b.value;
			if (!e) return;
			let t = document.createElement("canvas").getContext("2d");
			if (!t) return;
			let n = getComputedStyle(e);
			t.font = `${n.fontWeight} ${n.fontSize} ${n.fontFamily}`;
			let r = [
				"Rust",
				"C",
				"Python",
				"TypeScript",
				"ABT"
			], i = 0;
			for (let e of r) i = Math.max(i, t.measureText(e).width);
			x.value = `${Math.ceil(i + 12 + 20 + 4)}px`;
		});
		let S = y(!1);
		O(() => d.mode, () => {
			S.value = !1;
		});
		let w = t(() => {
			let e = d.transTarget;
			return d.mode === "trans" && (e === "python" || e === "typescript");
		});
		async function T() {
			let e = d.transTarget;
			!e || !d.onRunCode || (S.value = !0, await d.onRunCode(e));
		}
		let D = t(() => d.mode === "run" || d.mode === "debug" || d.mode === "replay" ? "Bytecode" : d.mode === "trans" ? _.value : ""), k = t(() => {
			if (d.mode === "trans") return d.transTarget;
		}), j = t(() => d.mode === "trans" && (d.transFiles?.length ?? 0) > 1), M = t(() => (d.projectFiles?.length ?? 0) > 1), N = t(() => d.mappedSourceFiles ? Array.from(d.mappedSourceFiles) : []);
		function P(e) {
			d.onOutputLineClick?.(d.selectedTransFile ?? "", e);
		}
		let F = t(() => (d.mode, d.bytecode ?? []));
		function I(e) {
			d.onSelectTransFile?.(d.transTarget, e);
		}
		let L = t(() => d.mode === "run" || d.mode === "debug" || d.mode === "replay" || S.value), R = t(() => d.mode === "run" || S.value ? "Output" : d.mode === "debug" || d.mode === "replay" ? "Debug Output" : "");
		function z() {
			d.onTrans();
		}
		function B(e) {
			f("loadExample", e);
		}
		return (t, u) => (v(), i("div", xr, [
			a("header", Sr, [a("div", Cr, [u[21] ||= a("h1", { class: "title" }, "Auto Playground", -1), c(qt, { onSelect: B })]), a("div", wr, [
				!l.isDebugging && !l.isReplayMode ? (v(), i("button", {
					key: 0,
					class: "toolbar-btn load-replay-btn",
					onClick: u[0] ||= (e) => t.$emit("loadReplay"),
					title: "Load Replay File"
				}, [...u[22] ||= [a("span", { class: "icon" }, "📂", -1), a("span", { class: "label" }, "Load Replay", -1)]])) : r("", !0),
				a("button", {
					class: "toolbar-btn share-btn",
					onClick: u[1] ||= (e) => t.$emit("share"),
					title: "Copy shareable link"
				}, [...u[23] ||= [a("svg", {
					width: "14",
					height: "14",
					viewBox: "0 0 24 24",
					fill: "none",
					stroke: "currentColor",
					"stroke-width": "2.5",
					"stroke-linecap": "round",
					"stroke-linejoin": "round"
				}, [
					a("path", { d: "M4 12v8a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-8" }),
					a("polyline", { points: "16 6 12 2 8 6" }),
					a("line", {
						x1: "12",
						y1: "2",
						x2: "12",
						y2: "15"
					})
				], -1), s(" Share ", -1)]]),
				a("button", {
					class: p(["toolbar-btn debug-btn", {
						active: l.isDebugging,
						exit: l.isDebugging
					}]),
					onClick: u[2] ||= (e) => l.isDebugging ? t.$emit("debugCommand", "stop") : d.onDebug(),
					disabled: l.isLoading || l.isReplayMode,
					title: l.isDebugging ? "Stop Debugging (Shift+F5)" : "Start Debugging"
				}, [u[24] ||= o("<svg width=\"14\" height=\"14\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2.5\" stroke-linecap=\"round\" stroke-linejoin=\"round\" data-v-9c6ad4e9><path d=\"M12 2a10 10 0 0 1 10 10\" data-v-9c6ad4e9></path><path d=\"M12 2a10 10 0 0 0-10 10\" data-v-9c6ad4e9></path><path d=\"M12 12l4-4\" data-v-9c6ad4e9></path><path d=\"M12 12l-4-4\" data-v-9c6ad4e9></path><path d=\"M12 12l4 4\" data-v-9c6ad4e9></path><path d=\"M12 12l-4 4\" data-v-9c6ad4e9></path></svg>", 1), s(" " + C(l.isDebugging ? "Exit Debug" : "Debug"), 1)], 10, Tr),
				l.isDebugging ? r("", !0) : (v(), i("button", {
					key: 1,
					class: "toolbar-btn run-btn",
					onClick: u[3] ||= (...e) => d.onRun && d.onRun(...e),
					disabled: l.isLoading || l.isReplayMode
				}, C(l.isLoading ? "Running..." : "Run (Ctrl+Enter)"), 9, Er)),
				l.isDebugging ? r("", !0) : (v(), i("div", {
					key: 2,
					class: p(["trans-split-btn", { disabled: l.isLoading || l.isReplayMode }]),
					title: "Transpile to target language"
				}, [a("button", {
					class: "trans-main",
					onClick: u[4] ||= (...e) => d.onTrans && d.onTrans(...e),
					disabled: l.isLoading || l.isReplayMode
				}, " Trans ", 8, Dr), a("div", {
					ref_key: "transDropdownEl",
					ref: b,
					class: "trans-dropdown",
					style: m({ width: x.value })
				}, [
					A(a("select", {
						"onUpdate:modelValue": u[5] ||= (e) => h.value = e,
						class: "trans-select",
						disabled: l.isLoading || l.isDebugging || l.isReplayMode,
						onChange: z
					}, [...u[25] ||= [o("<option value=\"rust\" data-v-9c6ad4e9>Rust</option><option value=\"c\" data-v-9c6ad4e9>C</option><option value=\"python\" data-v-9c6ad4e9>Python</option><option value=\"typescript\" data-v-9c6ad4e9>TypeScript</option><option value=\"abt\" data-v-9c6ad4e9>ABT</option>", 5)]], 40, Or), [[E, h.value]]),
					a("span", kr, C(_.value), 1),
					u[26] ||= a("span", { class: "trans-arrow" }, [a("svg", {
						width: "12",
						height: "12",
						viewBox: "0 0 24 24",
						fill: "none",
						stroke: "currentColor",
						"stroke-width": "2.5",
						"stroke-linecap": "round",
						"stroke-linejoin": "round"
					}, [a("polyline", { points: "6 9 12 15 18 9" })])], -1)
				], 4)], 2))
			])]),
			l.isDebugging || l.hasRecording ? (v(), n(Vn, {
				key: 0,
				"is-paused": l.isPaused,
				"is-recording": l.isRecording,
				"has-recording": l.hasRecording,
				onCommand: u[6] ||= (e) => t.$emit("debugCommand", e),
				onToggleRecord: u[7] ||= (e) => t.$emit("toggleRecord"),
				onExportRecording: u[8] ||= (e) => t.$emit("exportRecording")
			}, null, 8, [
				"is-paused",
				"is-recording",
				"has-recording"
			])) : r("", !0),
			l.isReplayMode ? (v(), n(Jn, {
				key: 1,
				"is-playing": l.isReplayPlaying,
				"current-index": l.replayCurrentIndex,
				"total-frames": l.replayTotalFrames,
				onPlay: u[9] ||= (e) => t.$emit("replayPlay"),
				onPause: u[10] ||= (e) => t.$emit("replayPause"),
				onStepForward: u[11] ||= (e) => t.$emit("replayStepForward"),
				onStepBackward: u[12] ||= (e) => t.$emit("replayStepBackward"),
				onSeek: u[13] ||= (e) => t.$emit("replaySeek", e)
			}, null, 8, [
				"is-playing",
				"current-index",
				"total-frames"
			])) : r("", !0),
			a("div", Ar, [a("div", jr, [a("div", { class: p(["editor-pane", { "with-preview": l.mode !== "editor" }]) }, [a("div", Mr, [l.isReplayMode ? (v(), i("span", Nr, "Replay")) : (v(), i("span", Pr, [u[27] ||= s("Auto ", -1), l.activeFile ? (v(), i("span", Fr, "· " + C(l.activeFile), 1)) : r("", !0)]))]), a("div", Ir, [M.value ? (v(), n(Xt, {
				key: 0,
				files: l.projectFiles,
				selected: l.activeFile || "",
				"mapped-files": N.value,
				onSelect: u[14] ||= (e) => t.$emit("selectFile", e)
			}, null, 8, [
				"files",
				"selected",
				"mapped-files"
			])) : r("", !0), c(Je, {
				"model-value": l.source,
				"onUpdate:modelValue": u[15] ||= (e) => t.$emit("update:source", e),
				"on-run": l.onRun,
				"is-debugging": l.isDebugging || l.isReplayMode,
				breakpoints: l.breakpoints,
				"current-debug-line": l.currentDebugLine,
				"highlighted-source-line": l.currentSourceLine,
				"selected-source-line": l.selectedSourceLine,
				"read-only": l.isReplayMode,
				onLineClick: u[16] ||= (e) => t.$emit("lineClick", e),
				onBreakpointsChange: u[17] ||= (e) => t.$emit("breakpointsChange", e),
				onHoverLine: u[18] ||= (e) => d.onHighlightLine?.(e),
				onHoverLineLeave: u[19] ||= (e) => d.onClearHighlight?.()
			}, null, 8, [
				"model-value",
				"on-run",
				"is-debugging",
				"breakpoints",
				"current-debug-line",
				"highlighted-source-line",
				"selected-source-line",
				"read-only"
			])])], 2), l.mode === "editor" ? r("", !0) : (v(), i("div", Lr, [a("div", Rr, [a("span", null, C(D.value), 1), w.value ? (v(), i("button", {
				key: 0,
				class: "run-code-btn",
				disabled: l.isLoading || l.isReplayMode,
				onClick: T
			}, " Run " + C(_.value), 9, zr)) : r("", !0)]), a("div", { class: p(["pane-body", { "with-file-tree": j.value }]) }, [l.mode === "run" || l.mode === "debug" || l.mode === "replay" ? (v(), n(Ht, {
				key: 0,
				bytecode: F.value,
				"bytecode-meta": l.bytecodeMeta,
				"current-ip": l.debugState?.ip,
				"selected-offsets": l.selectedOffsets,
				"highlighted-offsets": l.highlightedOffsets,
				onOffsetClick: u[20] ||= (e) => t.$emit("offsetClick", e)
			}, null, 8, [
				"bytecode",
				"bytecode-meta",
				"current-ip",
				"selected-offsets",
				"highlighted-offsets"
			])) : l.mode === "trans" ? (v(), i(e, { key: 1 }, [j.value ? (v(), n(Xt, {
				key: 0,
				files: l.transFiles || [],
				selected: l.selectedTransFile || "",
				onSelect: I
			}, null, 8, ["files", "selected"])) : r("", !0), c(gt, {
				code: l.transpiledCode,
				language: k.value,
				"highlight-lines": l.highlightLines,
				onLineClick: P
			}, null, 8, [
				"code",
				"language",
				"highlight-lines"
			])], 64)) : r("", !0)], 2)]))]), L.value ? (v(), i("div", Br, [a("div", Vr, [a("span", null, C(R.value), 1)]), a("div", Hr, [c(St, {
				class: "console-main",
				stdout: l.stdout,
				stderr: l.stderr,
				result: l.resultCode,
				"time-ms": l.timeMs
			}, null, 8, [
				"stdout",
				"stderr",
				"result",
				"time-ms"
			]), (l.isDebugging || l.isReplayMode) && l.debugState ? (v(), n(br, {
				key: 0,
				state: l.debugState
			}, null, 8, ["state"])) : r("", !0)])])) : r("", !0)])
		]));
	}
}), [["__scopeId", "data-v-9c6ad4e9"]]), Wr = "/api", Gr = "auto-playground:state", Kr = "// Welcome to Auto Playground!\nfn add(a int, b int) int {\n    a + b\n}\n\nlet result = add(3, 4)\nprint(result)";
function qr() {
	let e = window.location.hash;
	if (e.startsWith("#share=")) try {
		let t = atob(decodeURIComponent(e.slice(7))), n = JSON.parse(t);
		if (n.source) return n;
	} catch {}
	try {
		let e = localStorage.getItem(Gr);
		if (e) return JSON.parse(e);
	} catch {}
	return {};
}
function Jr(e) {
	try {
		localStorage.setItem(Gr, JSON.stringify(e));
	} catch {}
}
function Yr() {
	let e = qr(), n = y(e.source ?? Kr), r = y(""), i = y(""), a = y(""), o = y(0), s = y([]), c = y(null), l = y(!1), u = y(e.activeTab ?? "rust"), d = y(""), f = y(e.projectDir), p = y(e.projectFiles ?? []), m = y(e.activeFile ?? "");
	function h() {
		if (!m.value) return;
		let e = p.value.find((e) => e.path === m.value);
		e && (e.source = n.value);
	}
	function g(e) {
		if (e === m.value) return;
		h(), m.value = e;
		let t = p.value.find((t) => t.path === e);
		t && (n.value = t.source);
	}
	function _(e) {
		if (f.value) {
			h(), e.project_dir = f.value, e.files = p.value;
			let t = p.value.find((e) => e.path === "main.at");
			t && (e.source = t.source);
		}
		return e;
	}
	let v = y({}), b = y(null), x = y([]), S = y([]), C = y({
		message: "",
		visible: !1
	}), w = t(() => {
		let e = d.value;
		if (!e) return "";
		let t = v.value[e];
		return t ? t.files.find((e) => e.path === t.selectedFile)?.code ?? t.files[0]?.code ?? "" : "";
	}), T = t(() => {
		let e = d.value;
		return e ? v.value[e]?.files ?? [] : [];
	}), E = t(() => {
		let e = d.value;
		return e ? v.value[e]?.selectedFile ?? "" : "";
	}), D = t(() => m.value || ""), k = t(() => {
		let e = d.value, t = /* @__PURE__ */ new Map();
		if (!e) return t;
		let n = v.value[e];
		if (!n) return t;
		for (let e of n.files) {
			let r = n.fileSourceMaps[e.path] ?? [];
			for (let n of r) {
				let r = n.source_file || D.value;
				t.has(r) || t.set(r, /* @__PURE__ */ new Map());
				let i = t.get(r);
				i.has(n.source_line) || i.set(n.source_line, []);
				let a = i.get(n.source_line), o = a.find((t) => t.outputFile === e.path);
				o ? o.outputLines.includes(n.output_line) || o.outputLines.push(n.output_line) : a.push({
					outputFile: e.path,
					outputLines: [n.output_line]
				});
			}
		}
		return t;
	}), A = t(() => {
		let e = d.value, t = /* @__PURE__ */ new Map();
		if (!e) return t;
		let n = v.value[e];
		if (!n) return t;
		for (let e of n.files) {
			let r = n.fileSourceMaps[e.path] ?? [];
			for (let n of r) {
				let r = n.source_file || D.value;
				t.has(e.path) || t.set(e.path, /* @__PURE__ */ new Map()), t.get(e.path).set(n.output_line, {
					sourceFile: r,
					sourceLine: n.source_line
				});
			}
		}
		return t;
	}), j = t(() => {
		let e = d.value, t = /* @__PURE__ */ new Set();
		if (!e) return t;
		let n = v.value[e];
		if (!n) return t;
		for (let e of n.files) for (let r of n.fileSourceMaps[e.path] ?? []) t.add(r.source_file || D.value);
		return t;
	});
	function M() {
		b.value ? N(b.value) : (x.value = [], S.value = []);
	}
	function N(e) {
		b.value = e;
		let t = D.value, n = k.value.get(t)?.get(e) ?? [];
		S.value = n.map((e) => e.outputFile);
		let r = E.value;
		x.value = n.find((e) => e.outputFile === r)?.outputLines ?? [];
	}
	function P(e, t) {
		let n = A.value.get(e)?.get(t);
		if (!n) {
			I();
			return;
		}
		n.sourceFile && p.value.length > 0 && m.value !== n.sourceFile && g(n.sourceFile), b.value = n.sourceLine;
		let r = k.value.get(n.sourceFile)?.get(n.sourceLine) ?? [];
		S.value = r.map((e) => e.outputFile), x.value = r.find((t) => t.outputFile === e)?.outputLines ?? [];
	}
	function F(e, t) {
		return A.value.get(e)?.get(t)?.sourceFile;
	}
	function I() {
		b.value = null, x.value = [], S.value = [];
	}
	async function L() {
		l.value = !0, r.value = "", i.value = "", a.value = "", s.value = [], c.value = null;
		try {
			let e = _({ source: n.value }), t = await (await fetch(`${Wr}/run`, {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify(e)
			})).json();
			r.value = t.stdout || "", i.value = t.stderr || "", o.value = t.time_ms || 0, s.value = t.bytecode || [], c.value = t.meta ?? null, t.result !== void 0 && t.result !== null && t.result !== "" && (a.value = t.result);
		} catch (e) {
			i.value = `Network error: ${e.message}`;
		} finally {
			l.value = !1;
		}
	}
	async function R(e) {
		l.value = !0, r.value = "", i.value = "", a.value = "", o.value = 0;
		let t = v.value[e]?.files[0]?.code ?? "";
		if (!t.trim()) {
			i.value = `No ${e} code to run. Make sure the transpilation succeeded.`, l.value = !1;
			return;
		}
		try {
			if (e === "typescript") {
				let e = await Et(t);
				r.value = e.stdout, i.value = e.stderr, o.value = 0;
			} else {
				let n = await (await fetch(`${Wr}/run_code`, {
					method: "POST",
					headers: { "Content-Type": "application/json" },
					body: JSON.stringify({
						language: e,
						code: t
					})
				})).json();
				r.value = n.stdout || "", i.value = n.stderr || "", o.value = n.time_ms || 0, n.result !== void 0 && n.result !== null && n.result !== "" && (a.value = n.result);
			}
		} catch (e) {
			i.value = `Network error: ${e.message}`;
		} finally {
			l.value = !1;
		}
	}
	async function z(e) {
		l.value = !0;
		try {
			let t = _({
				source: n.value,
				target: e
			}), r = await (await fetch(`${Wr}/trans`, {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify(t)
			})).json(), i = r.files ?? [], a = {};
			for (let e of i) a[e.path] = e.source_map ?? r.source_map ?? [];
			let o = i[0]?.path ?? "";
			d.value = e, v.value[e] = {
				files: i,
				fileSourceMaps: a,
				selectedFile: o
			}, M();
		} catch (t) {
			d.value = e, v.value[e] = {
				files: [{
					path: "error.txt",
					code: `Error: ${t.message}`
				}],
				fileSourceMaps: { "error.txt": [] },
				selectedFile: "error.txt"
			}, M();
		} finally {
			l.value = !1;
		}
	}
	function B(e) {
		u.value = e, d.value = e, M();
	}
	function V(e, t) {
		let n = v.value[e];
		n && (n.selectedFile = t, M());
	}
	function H(e) {
		n.value = e.source, f.value = e.project_dir, p.value = e.files ?? [], m.value = e.files?.length ? "main.at" : "", r.value = "", i.value = "", a.value = "", s.value = [], b.value = null, x.value = [], S.value = [];
	}
	function U() {
		let e = JSON.stringify({
			source: n.value,
			activeTab: u.value,
			projectDir: f.value,
			projectFiles: p.value.length ? p.value : void 0,
			activeFile: m.value || void 0
		}), t = "#share=" + encodeURIComponent(btoa(e));
		return window.location.origin + window.location.pathname + t;
	}
	async function W() {
		let e = U(), t = !1;
		try {
			await navigator.clipboard.writeText(e), t = !0;
		} catch {
			let n = document.createElement("textarea");
			n.value = e, document.body.appendChild(n), n.select();
			try {
				t = document.execCommand("copy");
			} catch {}
			document.body.removeChild(n);
		}
		C.value = {
			message: t ? "Share link copied to clipboard!" : "Failed to copy link",
			visible: !0
		}, setTimeout(() => {
			C.value.visible = !1;
		}, 2500);
	}
	return O(n, () => {
		v.value = {}, d.value && (d.value = "", x.value = [], S.value = []);
	}), O([
		n,
		u,
		f,
		p,
		m
	], ([e, t, n, r, i]) => {
		Jr({
			source: e,
			activeTab: t,
			projectDir: n,
			projectFiles: r,
			activeFile: i
		});
	}, { deep: !0 }), {
		source: n,
		stdout: r,
		stderr: i,
		resultCode: a,
		timeMs: o,
		bytecode: s,
		bytecodeMeta: c,
		isLoading: l,
		activeTab: u,
		transpiledCode: w,
		transpileTarget: d,
		projectDir: f,
		projectFiles: p,
		activeFile: m,
		transFiles: T,
		selectedTransFile: E,
		highlightedSourceLine: b,
		highlightedOutputLines: x,
		highlightedOutputFiles: S,
		mappedSourceFiles: j,
		shareToast: C,
		run: L,
		runCode: R,
		transpile: z,
		switchTab: B,
		selectTransFile: V,
		selectFile: g,
		loadExample: H,
		highlightSourceLine: N,
		highlightOutputLine: P,
		getSourceFileForOutputLine: F,
		clearHighlight: I,
		share: W
	};
}
//#endregion
//#region src/composables/useDebugger.ts
function Xr() {
	let e = y(null), n = y(!1), r = y(!1), i = y([]), a = y(null), o = y(null), s = y(null), c = y(!1), l = y(null), u = t(() => {
		let e = {};
		for (let t of i.value) t.line !== void 0 && (e[t.line] || (e[t.line] = []), e[t.line].push(t.offset));
		return e;
	}), d = t(() => {
		let e = {};
		for (let t of i.value) t.line !== void 0 && (e[t.offset] = t.line);
		return e;
	});
	function f(t, i = []) {
		if (e.value) return;
		let a = window.location.protocol === "https:" ? "wss:" : "ws:", o = new WebSocket(`${a}//${window.location.host}/api/debug/ws`);
		o.onopen = () => {
			n.value = !0, r.value = !0, o.send(JSON.stringify({
				type: "debug.start",
				source: t
			})), i.length > 0 && o.send(JSON.stringify({
				type: "breakpoints.set",
				lines: i
			}));
		}, o.onmessage = (e) => {
			p(JSON.parse(e.data));
		}, o.onerror = (e) => {
			s.value = "WebSocket error", console.error("Debug WS error:", e);
		}, o.onclose = () => {
			n.value = !1, r.value = !1, e.value = null;
		}, e.value = o;
	}
	function p(e) {
		switch (e.type) {
			case "bytecode":
				i.value = e.lines || [], a.value = e.meta ?? null, c.value && l.value && (l.value.bytecode = e.lines || [], l.value.meta = e.meta ?? null);
				break;
			case "state":
				o.value = e.data, c.value && l.value && l.value.events.push({
					type: "state",
					state: e.data
				}), (e.data.status === "finished" || e.data.status === "error") && (r.value = !1);
				break;
			case "error":
				s.value = e.message, r.value = !1;
				break;
		}
	}
	function m(t) {
		e.value?.readyState === WebSocket.OPEN && e.value.send(JSON.stringify({
			type: "command",
			cmd: t
		})), c.value && l.value && l.value.events.push({
			type: "command",
			cmd: t
		});
	}
	function h(t) {
		e.value?.readyState === WebSocket.OPEN && e.value.send(JSON.stringify({
			type: "breakpoints.set",
			lines: t
		})), c.value && l.value && l.value.events.push({
			type: "breakpoints",
			lines: t
		});
	}
	function g() {
		m("stop"), e.value?.close(), e.value = null, r.value = !1, o.value = null, i.value = [], a.value = null, s.value = null;
	}
	function _(e, t) {
		l.value = {
			version: 1,
			createdAt: (/* @__PURE__ */ new Date()).toISOString(),
			source: e,
			initialBreakpoints: [...t],
			bytecode: [],
			events: []
		}, c.value = !0;
	}
	function v() {
		return c.value = !1, l.value;
	}
	function b() {
		if (!l.value) return;
		let e = new Blob([JSON.stringify(l.value, null, 2)], { type: "application/json" }), t = URL.createObjectURL(e), n = document.createElement("a");
		n.href = t, n.download = `replay_${Date.now()}.autoreplay`, n.click(), URL.revokeObjectURL(t);
	}
	return {
		isConnected: n,
		isDebugging: r,
		bytecode: i,
		meta: a,
		state: o,
		error: s,
		lineToOffsets: u,
		offsetToLine: d,
		connect: f,
		sendCommand: m,
		setBreakpoints: h,
		stop: g,
		isRecording: c,
		recording: l,
		startRecording: _,
		stopRecording: v,
		exportRecording: b
	};
}
//#endregion
//#region src/composables/useReplayPlayer.ts
function Zr() {
	let e = y(!1), n = y(null), r = y(0), i = y(!1), a = null, o = t(() => n.value?.events ?? []), s = t(() => o.value.filter((e) => e.type === "state").map((e, t) => ({
		...e,
		frameIndex: t
	}))), c = t(() => s.value.length), l = t(() => {
		if (!e.value || !n.value) return null;
		let t = o.value[r.value];
		if (t?.type === "state") return t.state;
		for (let e = r.value; e >= 0; e--) {
			let t = o.value[e];
			if (t.type === "state") return t.state;
		}
		return null;
	}), u = t(() => n.value?.bytecode ?? []), d = t(() => n.value?.meta ?? null), f = t(() => {
		let e = {};
		for (let t of u.value) t.line !== void 0 && (e[t.line] || (e[t.line] = []), e[t.line].push(t.offset));
		return e;
	}), p = t(() => {
		let e = {};
		for (let t of u.value) t.line !== void 0 && (e[t.offset] = t.line);
		return e;
	});
	function m(t) {
		h(), n.value = t, e.value = !0, r.value = 0;
	}
	function h() {
		_(), e.value = !1, n.value = null, r.value = 0;
	}
	function g() {
		i.value || (i.value = !0, a = setInterval(() => {
			if (r.value >= o.value.length - 1) {
				_();
				return;
			}
			r.value++;
		}, 800));
	}
	function _() {
		i.value = !1, a &&= (clearInterval(a), null);
	}
	function v() {
		_(), r.value < o.value.length - 1 && r.value++;
	}
	function b() {
		_(), r.value > 0 && r.value--;
	}
	function x(e) {
		_(), r.value = Math.max(0, Math.min(o.value.length - 1, e));
	}
	return {
		isActive: e,
		recording: n,
		currentIndex: r,
		isPlaying: i,
		currentState: l,
		bytecode: u,
		meta: d,
		lineToOffsets: f,
		offsetToLine: p,
		totalFrames: c,
		load: m,
		stop: h,
		play: g,
		pause: _,
		stepForward: v,
		stepBackward: b,
		seek: x
	};
}
//#endregion
//#region src/AutoPlaygroundFull.vue
var Qr = /* @__PURE__ */ l({
	__name: "AutoPlaygroundFull",
	setup(n) {
		let { source: r, stdout: o, stderr: s, resultCode: l, timeMs: u, bytecode: d, bytecodeMeta: f, isLoading: m, activeTab: h, transpiledCode: b, transFiles: x, selectedTransFile: S, projectFiles: T, activeFile: E, highlightedOutputLines: D, highlightedSourceLine: k, mappedSourceFiles: A, run: j, transpile: M, runCode: N, selectTransFile: P, selectFile: F, loadExample: I, highlightOutputLine: L, share: R, shareToast: z } = Yr(), B = Xr(), V = Zr(), H = y([]), U = y("editor"), W = y("rust"), ee = t(() => V.isActive.value ? V.currentState.value : B.state.value), te = t(() => V.isActive.value ? V.bytecode.value : B.bytecode.value), G = t(() => V.isActive.value ? V.meta.value : B.meta.value), K = t(() => U.value === "run" ? d.value : te.value), q = t(() => {
			let e = {};
			for (let t of K.value) t.line !== void 0 && (e[t.line] || (e[t.line] = []), e[t.line].push(t.offset));
			return e;
		}), J = t(() => {
			let e = {};
			for (let t of K.value) t.line !== void 0 && (e[t.offset] = t.line);
			return e;
		}), Y = y(null), ne = t(() => k.value ? q.value[k.value] : void 0), re = t(() => Y.value ? q.value[Y.value] : void 0);
		function ie(e) {
			k.value = e;
		}
		function X(e) {
			Y.value = e;
		}
		function ae() {
			Y.value = null;
		}
		O(() => B.state.value, (e) => {
			e?.status === "finished" && (o.value = e.stdout || "", l.value = e.result || "", s.value = e.stderr || "");
		}), O(() => B.isDebugging.value, (e) => {
			!e && U.value === "debug" && B.state.value?.status !== "finished" && (U.value = "editor");
		}), O(() => V.isActive.value, (e) => {
			!e && U.value === "replay" && (U.value = "editor");
		});
		async function oe() {
			U.value = "run", o.value = "", s.value = "", l.value = "", await j();
		}
		async function se() {
			U.value = "trans", await M(W.value), h.value = W.value;
		}
		async function Z(e) {
			await N(e);
		}
		function ce() {
			B.isDebugging.value || (U.value = "debug", V.stop(), B.connect(r.value, H.value));
		}
		function Q() {
			B.isRecording.value ? B.stopRecording() : B.startRecording(r.value, H.value);
		}
		function le(e) {
			B.sendCommand(e);
		}
		function ue(e) {
			let t = J.value[e];
			t && ie(t);
		}
		function de(e) {
			H.value = e, B.setBreakpoints(e);
		}
		async function fe() {
			let e = document.createElement("input");
			e.type = "file", e.accept = ".autoreplay,.json", e.onchange = async () => {
				let t = e.files?.[0];
				if (t) try {
					let e = await t.text(), n = JSON.parse(e);
					B.stop(), V.load(n), U.value = "replay";
				} catch (e) {
					alert("Failed to load replay file: " + e.message);
				}
			}, e.click();
		}
		function pe(e) {
			I(e), U.value = "editor";
		}
		function me(e) {
			if (V.isActive.value) {
				switch (e.key) {
					case "ArrowRight":
						e.preventDefault(), V.stepForward();
						break;
					case "ArrowLeft":
						e.preventDefault(), V.stepBackward();
						break;
					case " ":
						e.preventDefault(), V.isPlaying.value ? V.pause() : V.play();
						break;
				}
				return;
			}
			if (B.isDebugging.value) switch (e.key) {
				case "F5":
					e.preventDefault(), le(e.shiftKey ? "stop" : "continue");
					break;
				case "F10":
					e.preventDefault(), le("step_over");
					break;
				case "F11":
					e.preventDefault(), le(e.shiftKey ? "step_out" : "step");
					break;
			}
		}
		return g(() => {
			window.addEventListener("keydown", me), window.__loadReplayForTest__ = (e) => {
				V.load(e), U.value = "replay";
			};
		}), _(() => {
			window.removeEventListener("keydown", me);
		}), (t, n) => (v(), i(e, null, [c(Ur, {
			source: w(r),
			"is-loading": w(m),
			mode: U.value,
			"trans-target": W.value,
			"onUpdate:transTarget": n[0] ||= (e) => W.value = e,
			stdout: w(o),
			stderr: w(s),
			"result-code": w(l),
			"time-ms": w(u),
			"transpiled-code": w(b),
			"trans-files": w(x),
			"selected-trans-file": w(S),
			"project-files": w(T),
			"active-file": w(E),
			"mapped-source-files": w(A),
			"highlight-lines": w(D),
			"on-run": oe,
			"on-trans": se,
			"on-run-code": Z,
			"on-debug": ce,
			"on-select-trans-file": w(P),
			"on-output-line-click": w(L),
			"is-debugging": w(B).isDebugging.value,
			"is-paused": w(B).state.value?.status === "paused",
			"is-recording": w(B).isRecording.value,
			"has-recording": !!w(B).recording.value,
			bytecode: K.value,
			"bytecode-meta": U.value === "run" ? w(f) : G.value,
			"debug-state": ee.value,
			"current-source-line": Y.value,
			"highlighted-offsets": re.value,
			"selected-offsets": ne.value,
			"selected-source-line": w(k),
			breakpoints: H.value,
			"current-debug-line": ee.value?.line ?? null,
			"is-replay-mode": w(V).isActive.value,
			"replay-current-index": w(V).currentIndex.value,
			"replay-total-frames": w(V).totalFrames.value,
			"is-replay-playing": w(V).isPlaying.value,
			"onUpdate:source": n[1] ||= (e) => r.value = e,
			onLoadExample: pe,
			onSelectFile: w(F),
			onShare: w(R),
			onDebugCommand: le,
			onToggleRecord: Q,
			onExportRecording: w(B).exportRecording,
			onLineClick: ie,
			"on-highlight-line": X,
			"on-clear-highlight": ae,
			onOffsetClick: ue,
			onBreakpointsChange: de,
			onLoadReplay: fe,
			onReplayPlay: w(V).play,
			onReplayPause: w(V).pause,
			onReplayStepForward: w(V).stepForward,
			onReplayStepBackward: w(V).stepBackward,
			onReplaySeek: w(V).seek
		}, null, 8, /* @__PURE__ */ "source.is-loading.mode.trans-target.stdout.stderr.result-code.time-ms.transpiled-code.trans-files.selected-trans-file.project-files.active-file.mapped-source-files.highlight-lines.on-select-trans-file.on-output-line-click.is-debugging.is-paused.is-recording.has-recording.bytecode.bytecode-meta.debug-state.current-source-line.highlighted-offsets.selected-offsets.selected-source-line.breakpoints.current-debug-line.is-replay-mode.replay-current-index.replay-total-frames.is-replay-playing.onSelectFile.onShare.onExportRecording.onReplayPlay.onReplayPause.onReplayStepForward.onReplayStepBackward.onReplaySeek".split(".")), a("div", { class: p(["toast", { visible: w(z).visible }]) }, C(w(z).message), 3)], 64));
	}
}), $r = { class: "nx-sidebar" }, ei = { class: "nx-search" }, ti = ["value"], ni = { class: "nx-tree" }, ri = ["onClick"], ii = ["title"], ai = { class: "nx-count" }, oi = { class: "nx-notes" }, si = ["title", "onClick"], ci = {
	key: 0,
	class: "nx-empty"
}, li = /* @__PURE__ */ $(/* @__PURE__ */ l({
	__name: "NotesSidebar",
	props: /* @__PURE__ */ d({
		groups: {},
		searching: { type: Boolean },
		activeNoteId: {}
	}, {
		query: { default: "" },
		queryModifiers: {}
	}),
	emits: /* @__PURE__ */ d(["select"], ["update:query"]),
	setup(t, { emit: n }) {
		let o = t, s = T(t, "query"), l = n, u = y(/* @__PURE__ */ new Set());
		function d(e) {
			return o.searching ? !0 : u.value.has(e);
		}
		function f(e) {
			let t = new Set(u.value);
			t.has(e) ? t.delete(e) : t.add(e), u.value = t;
		}
		return O(() => o.activeNoteId, (e) => {
			if (!e) return;
			let t = o.groups.find((t) => t.notes.some((t) => t.id === e));
			if (t && !u.value.has(t.id)) {
				let e = new Set(u.value);
				e.add(t.id), u.value = e;
			}
			requestAnimationFrame(() => {
				document.querySelector(".nx-note-btn.active")?.scrollIntoView({ block: "nearest" });
			});
		}), (n, o) => (v(), i("aside", $r, [a("div", ei, [c(w(Te), {
			size: 14,
			class: "nx-search-icon"
		}), a("input", {
			class: "nx-search-input",
			type: "text",
			placeholder: "搜索标题或标签…",
			value: s.value,
			onInput: o[0] ||= (e) => s.value = e.target.value
		}, null, 40, ti)]), a("div", ni, [(v(!0), i(e, null, b(t.groups, (n) => (v(), i("section", {
			key: n.id,
			class: "nx-group"
		}, [a("button", {
			class: "nx-group-head",
			onClick: (e) => f(n.id)
		}, [
			c(w(me), {
				size: 13,
				class: p(["nx-chev", { open: d(n.id) }])
			}, null, 8, ["class"]),
			a("span", {
				class: "nx-group-title",
				title: `${n.id} · ${n.source}`
			}, C(n.title), 9, ii),
			a("span", ai, C(n.notes.length), 1)
		], 8, ri), A(a("ul", oi, [(v(!0), i(e, null, b(n.notes, (e) => (v(), i("li", { key: e.id }, [a("button", {
			class: p(["nx-note-btn", { active: e.id === t.activeNoteId }]),
			title: e.id,
			onClick: (t) => l("select", e.id)
		}, C(e.title), 11, si)]))), 128))], 512), [[D, d(n.id)]])]))), 128)), t.groups.length === 0 ? (v(), i("p", ci, "无匹配笔记")) : r("", !0)])]));
	}
}), [["__scopeId", "data-v-b587c4b7"]]);
//#endregion
//#region src/composables/useNotes.ts
function ui(e = {}) {
	let n = e.base ?? "/playground-data/notes.json", r = S(null), i = y(!1), a = y(null), o = t(() => r.value?.groups ?? []), s = t(() => o.value.flatMap((e) => e.notes.map((t) => ({
		note: t,
		group: e
	})))), c = t(() => {
		let e = /* @__PURE__ */ new Map();
		for (let t of s.value) e.set(t.note.id, t);
		return e;
	});
	async function l(e = n) {
		i.value = !0, a.value = null;
		try {
			let t = await fetch(e);
			if (!t.ok) throw Error(`HTTP ${t.status} ${t.statusText}`);
			let n = JSON.parse(await t.text());
			if (!n || !Array.isArray(n.groups)) throw Error("invalid manifest: groups missing");
			r.value = n;
		} catch (e) {
			r.value = null, a.value = e instanceof Error ? e.message : String(e);
		} finally {
			i.value = !1;
		}
	}
	e.immediate !== !1 && typeof window < "u" && l();
	function u(e) {
		let t = e.trim().toLowerCase();
		return t ? s.value.filter(({ note: e }) => e.title.toLowerCase().includes(t) || e.tags.some((e) => e.toLowerCase().includes(t))) : s.value;
	}
	return {
		groups: o,
		flatNotes: s,
		byId: c,
		isLoading: i,
		error: a,
		fetchNotes: l,
		search: u
	};
}
//#endregion
//#region src/components/NotesExplorer.vue?vue&type=script&setup=true&lang.ts
var di = { class: "notes-explorer" }, fi = { class: "nx-main" }, pi = {
	key: 0,
	class: "nx-state"
}, mi = {
	key: 1,
	class: "nx-state nx-error"
}, hi = {
	key: 2,
	class: "nx-state"
}, gi = {
	key: 3,
	class: "nx-state"
}, _i = {
	key: 4,
	class: "nx-note"
}, vi = { class: "nx-note-header" }, yi = { class: "nx-title-row" }, bi = { class: "nx-note-title" }, xi = {
	key: 0,
	class: "nx-kind-chip"
}, Si = { class: "nx-meta-row" }, Ci = ["href", "title"], wi = { class: "nx-source-path" }, Ti = {
	key: 0,
	class: "nx-standalone-warn",
	title: "含 import/use 顶层声明，不可独立运行"
}, Ei = {
	key: 0,
	class: "nx-desc"
}, Di = "#/notes/", Oi = /* @__PURE__ */ $(/* @__PURE__ */ l({
	__name: "NotesExplorer",
	props: {
		base: { default: "/playground-data/notes.json" },
		apiBase: { default: "" },
		repoBase: { default: "https://github.com/auto-stack/auto-lang" }
	},
	setup(e) {
		let o = e, { groups: l, flatNotes: u, byId: d, isLoading: f, error: m, fetchNotes: _, search: b } = ui({ base: o.base }), x = y(""), S = y(null), T = y(null), E = t(() => S.value ? d.value.get(S.value) ?? null : null), D = t(() => {
			if (!x.value.trim()) return l.value;
			let e = /* @__PURE__ */ new Map();
			for (let { note: t, group: n } of b(x.value)) e.has(n.id) || e.set(n.id, []), e.get(n.id).push(t);
			return l.value.map((t) => ({
				...t,
				notes: e.get(t.id) ?? []
			})).filter((e) => e.notes.length > 0);
		}), k = t(() => {
			let e = E.value?.note;
			return e ? `${o.repoBase.replace(/\/$/, "")}/blob/master/${e.sourcePath}` : "";
		}), A = {
			single: "单文件",
			project: "项目",
			fence: "书页围栏"
		}, j = t(() => A[E.value?.note.kind ?? ""] ?? E.value?.note.kind ?? ""), M = t(() => {
			let e = E.value?.note;
			return e ? e.code ?? e.files?.find((e) => e.path === "main.at")?.content ?? "" : "";
		});
		function N(e) {
			S.value = e;
		}
		function P() {
			if (typeof window > "u") return null;
			let e = window.location.hash;
			if (!e.startsWith(Di)) return null;
			try {
				return decodeURIComponent(e.slice(8));
			} catch {
				return null;
			}
		}
		function F(e) {
			if (typeof window > "u") return;
			let t = Di + encodeURIComponent(e).replace(/%2F/gi, "/");
			window.location.hash !== t && window.history.replaceState(null, "", t);
		}
		O(u, (e) => {
			if (S.value || e.length === 0) return;
			let t = P();
			S.value = t && d.value.has(t) ? t : e[0].note.id;
		}), O(S, (e) => {
			e && F(e);
		});
		function I() {
			let e = P();
			e && d.value.has(e) && e !== S.value && (S.value = e);
		}
		function L(e) {
			let t = e instanceof HTMLElement ? e : null;
			return t ? !!t.closest("input, textarea, select, [contenteditable=\"true\"], .cm-editor") : !1;
		}
		function R() {
			return D.value.flatMap((e) => e.notes.map((e) => e.id));
		}
		function z(e) {
			let t = R();
			if (t.length === 0) return;
			let n = S.value ? t.indexOf(S.value) : -1;
			S.value = t[n === -1 ? 0 : Math.min(t.length - 1, Math.max(0, n + e))];
		}
		function B(e) {
			if ((e.ctrlKey || e.metaKey) && e.key === "Enter") {
				e.preventDefault(), T.value?.run();
				return;
			}
			if (e.key === "ArrowUp" || e.key === "ArrowDown") {
				if (L(e.target)) return;
				e.preventDefault(), z(e.key === "ArrowDown" ? 1 : -1);
			}
		}
		return g(() => {
			window.addEventListener("hashchange", I), window.addEventListener("keydown", B);
		}), h(() => {
			window.removeEventListener("hashchange", I), window.removeEventListener("keydown", B);
		}), (t, o) => (v(), i("div", di, [c(li, {
			class: "nx-side",
			groups: D.value,
			searching: !!x.value.trim(),
			"active-note-id": S.value,
			query: x.value,
			"onUpdate:query": o[0] ||= (e) => x.value = e,
			onSelect: N
		}, null, 8, [
			"groups",
			"searching",
			"active-note-id",
			"query"
		]), a("main", fi, [w(f) ? (v(), i("div", pi, "正在加载笔记清单…")) : w(m) ? (v(), i("div", mi, [a("p", null, "笔记清单加载失败：" + C(w(m)), 1), a("button", {
			class: "nx-retry",
			onClick: o[1] ||= (e) => w(_)()
		}, [c(w(we), { size: 13 }), o[2] ||= s(" 重试 ", -1)])])) : w(u).length === 0 ? (v(), i("div", hi, "笔记清单为空。")) : E.value ? (v(), i("article", _i, [a("header", vi, [
			a("div", yi, [
				a("h2", bi, C(E.value.note.title), 1),
				a("span", { class: p(["nx-badge", `t-${E.value.note.sourceType}`]) }, C(E.value.note.sourceType), 3),
				E.value.note.kind === "single" ? r("", !0) : (v(), i("span", xi, C(j.value), 1))
			]),
			a("div", Si, [a("a", {
				class: "nx-source-chip",
				href: k.value,
				target: "_blank",
				rel: "noopener",
				title: E.value.note.sourcePath
			}, [
				c(w(xe), { size: 12 }),
				a("span", wi, C(E.value.note.sourcePath), 1),
				c(w(be), { size: 11 })
			], 8, Ci), E.value.note.standalone ? r("", !0) : (v(), i("span", Ti, " 依赖模块 "))]),
			E.value.note.description ? (v(), i("details", Ei, [o[3] ||= a("summary", null, "说明", -1), a("p", null, C(E.value.note.description), 1)])) : r("", !0)
		]), (v(), n(jn, {
			ref_key: "card",
			ref: T,
			key: E.value.note.id,
			code: M.value,
			"api-base": e.apiBase,
			"note-id": E.value.note.id,
			height: "auto"
		}, null, 8, [
			"code",
			"api-base",
			"note-id"
		]))])) : (v(), i("div", gi, "从左侧选择一条笔记开始浏览。"))])]));
	}
}), [["__scopeId", "data-v-7463ff22"]]), ki = { class: "expected-output-panel" }, Ai = {
	key: 0,
	class: "eop-banner pending"
}, ji = {
	key: 1,
	class: "eop-banner match"
}, Mi = {
	key: 2,
	class: "eop-banner diff"
}, Ni = {
	key: 3,
	class: "eop-lines"
}, Pi = { class: "eop-ln" }, Fi = { class: "eop-text" }, Ii = {
	key: 0,
	class: "eop-empty"
}, Li = {
	key: 4,
	class: "eop-lines"
}, Ri = {
	key: 0,
	class: "eop-line same"
}, zi = { class: "eop-ln" }, Bi = { class: "eop-text" }, Vi = {
	key: 0,
	class: "eop-line minus"
}, Hi = { class: "eop-ln" }, Ui = { class: "eop-text" }, Wi = {
	key: 1,
	class: "eop-line plus"
}, Gi = { class: "eop-ln" }, Ki = { class: "eop-text" }, qi = /* @__PURE__ */ $(/* @__PURE__ */ l({
	__name: "ExpectedOutputPanel",
	props: {
		expected: {},
		actual: {},
		hasRun: { type: Boolean }
	},
	setup(n) {
		let o = n;
		function s(e) {
			let t = e.replace(/\r\n/g, "\n").split("\n").map((e) => e.replace(/[ \t]+$/, ""));
			for (; t.length > 0 && t[t.length - 1] === "";) t.pop();
			return t;
		}
		let l = t(() => s(o.expected)), u = t(() => s(o.actual)), d = t(() => o.hasRun ? f(l.value, u.value) ? "match" : "diff" : "pending");
		function f(e, t) {
			if (e.length !== t.length) return !1;
			for (let n = 0; n < e.length; n++) if (e[n] !== t[n]) return !1;
			return !0;
		}
		let p = t(() => Math.max(l.value.length, u.value.length)), m = t(() => {
			let e = 0;
			for (let t = 0; t < p.value; t++) h(l.value, t) !== h(u.value, t) && e++;
			return e;
		});
		function h(e, t) {
			return t < e.length ? e[t] : void 0;
		}
		return (t, n) => (v(), i("div", ki, [d.value === "pending" ? (v(), i("div", Ai, [c(w(ge), { size: 13 }), n[0] ||= a("span", null, "期望输出（运行后在此对照实际结果）", -1)])) : d.value === "match" ? (v(), i("div", ji, [c(w(he), { size: 13 }), a("span", null, "输出一致（" + C(l.value.length) + " 行）", 1)])) : (v(), i("div", Mi, [c(w(_e), { size: 13 }), a("span", null, "输出有差异（" + C(m.value) + " / " + C(Math.max(l.value.length, u.value.length)) + " 行不一致）", 1)])), d.value === "pending" ? (v(), i("div", Ni, [(v(!0), i(e, null, b(l.value, (e, t) => (v(), i("div", {
			key: t,
			class: "eop-line"
		}, [a("span", Pi, C(t + 1), 1), a("span", Fi, C(e), 1)]))), 128)), l.value.length === 0 ? (v(), i("div", Ii, "（期望输出为空）")) : r("", !0)])) : (v(), i("div", Li, [(v(!0), i(e, null, b(p.value, (t) => (v(), i(e, { key: t }, [h(l.value, t - 1) === h(u.value, t - 1) ? (v(), i("div", Ri, [
			a("span", zi, C(t), 1),
			n[1] ||= a("span", { class: "eop-sign" }, "·", -1),
			a("span", Bi, C(h(l.value, t - 1)), 1)
		])) : (v(), i(e, { key: 1 }, [t - 1 < l.value.length ? (v(), i("div", Vi, [
			a("span", Hi, C(t), 1),
			n[2] ||= a("span", { class: "eop-sign" }, "-", -1),
			a("span", Ui, C(l.value[t - 1]), 1)
		])) : r("", !0), t - 1 < u.value.length ? (v(), i("div", Wi, [
			a("span", Gi, C(t), 1),
			n[3] ||= a("span", { class: "eop-sign" }, "+", -1),
			a("span", Ki, C(u.value[t - 1]), 1)
		])) : r("", !0)], 64))], 64))), 128))]))]));
	}
}), [["__scopeId", "data-v-49948f1e"]]);
//#endregion
export { Mn as AutoPlayground, Qr as AutoPlaygroundFull, Ht as BytecodePanel, Je as CodeEditor, gt as CodePreview, St as ConsoleOutput, qt as ExampleSelector, qi as ExpectedOutputPanel, Oi as NotesExplorer, li as NotesSidebar, jn as PlaygroundCard, Ur as PlaygroundLayout, Ye as ScrollArea, Ft as SnippetRunner, Fe as autoLanguage, Xr as useDebugger, ui as useNotes, kt as usePlayground, Yr as usePlaygroundFull, Zr as useReplayPlayer };
