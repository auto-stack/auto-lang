import { Fragment as e, computed as t, createBlock as n, createCommentVNode as r, createElementBlock as i, createElementVNode as a, createStaticVNode as o, createTextVNode as s, createVNode as c, defineComponent as l, h as u, mergeModels as d, nextTick as f, normalizeClass as p, normalizeStyle as m, onBeforeUnmount as h, onMounted as g, onUnmounted as _, openBlock as v, ref as y, renderList as b, renderSlot as x, shallowRef as S, toDisplayString as C, unref as w, useModel as T, vModelSelect as E, vShow as D, watch as O, withCtx as k, withDirectives as A } from "vue";
import { Compartment as j, EditorState as M, RangeSetBuilder as N, StateEffect as P, StateField as F } from "@codemirror/state";
import { Decoration as I, EditorView as L, GutterMarker as R, gutter as z, highlightActiveLine as B, keymap as V, lineNumbers as H } from "@codemirror/view";
import { defaultKeymap as U, history as W, historyKeymap as G, indentWithTab as K } from "@codemirror/commands";
import { oneDark as q } from "@codemirror/theme-one-dark";
import { StreamLanguage as ee } from "@codemirror/language";
//#region \0rolldown/runtime.js
var te = Object.create, ne = Object.defineProperty, re = Object.getOwnPropertyDescriptor, J = Object.getOwnPropertyNames, ie = Object.getPrototypeOf, Y = Object.prototype.hasOwnProperty, X = (e, t) => () => (t || (e((t = { exports: {} }).exports, t), e = null), t.exports), Z = (e, t, n, r) => {
	if (t && typeof t == "object" || typeof t == "function") for (var i = J(t), a = 0, o = i.length, s; a < o; a++) s = i[a], !Y.call(e, s) && s !== n && ne(e, s, {
		get: ((e) => t[e]).bind(null, s),
		enumerable: !(r = re(t, s)) || r.enumerable
	});
	return e;
}, ae = (e, t, n) => (n = e == null ? {} : te(ie(e)), Z(t || !e || !e.__esModule ? ne(n, "default", {
	value: e,
	enumerable: !0
}) : n, e)), oe = (e) => e.replace(/([a-z0-9])([A-Z])/g, "$1-$2").toLowerCase(), se = {
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
	...se,
	width: e || se.width,
	height: e || se.height,
	stroke: r || se.stroke,
	"stroke-width": n ? Number(t) * 24 / Number(e) : t,
	class: ["lucide", `lucide-${oe(a ?? "icon")}`],
	...s
}, [...i.map((e) => u(...e)), ...c.default ? [c.default()] : []]), Q = (e, t) => (n, { slots: r }) => u(ce, {
	...n,
	iconNode: t,
	name: e
}, r), le = Q("AppWindowIcon", [
	["rect", {
		x: "2",
		y: "4",
		width: "20",
		height: "16",
		rx: "2",
		key: "izxlao"
	}],
	["path", {
		d: "M10 4v4",
		key: "pp8u80"
	}],
	["path", {
		d: "M2 8h20",
		key: "d11cs7"
	}],
	["path", {
		d: "M6 4v4",
		key: "1svtjw"
	}]
]), ue = Q("ArrowDownIcon", [["path", {
	d: "M12 5v14",
	key: "s699le"
}], ["path", {
	d: "m19 12-7 7-7-7",
	key: "1idqje"
}]]), de = Q("ArrowUpIcon", [["path", {
	d: "m5 12 7-7 7 7",
	key: "hav0vg"
}], ["path", {
	d: "M12 19V5",
	key: "x0mq9r"
}]]), fe = Q("BugIcon", [
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
]), pe = Q("CheckIcon", [["path", {
	d: "M20 6 9 17l-5-5",
	key: "1gmf2c"
}]]), me = Q("ChevronDownIcon", [["path", {
	d: "m6 9 6 6 6-6",
	key: "qrunsl"
}]]), he = Q("ChevronRightIcon", [["path", {
	d: "m9 18 6-6-6-6",
	key: "mthhwq"
}]]), ge = Q("CircleCheckIcon", [["circle", {
	cx: "12",
	cy: "12",
	r: "10",
	key: "1mglay"
}], ["path", {
	d: "m9 12 2 2 4-4",
	key: "dzmm74"
}]]), _e = Q("CircleDashedIcon", [
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
]), ve = Q("CircleXIcon", [
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
]), ye = Q("CodeXmlIcon", [
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
]), be = Q("CopyIcon", [["rect", {
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
}]]), xe = Q("ExternalLinkIcon", [
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
]), Se = Q("FileCodeIcon", [
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
]), Ce = Q("LoaderCircleIcon", [["path", {
	d: "M21 12a9 9 0 1 1-6.219-8.56",
	key: "13zald"
}]]), we = Q("LockIcon", [["rect", {
	width: "18",
	height: "11",
	x: "3",
	y: "11",
	rx: "2",
	ry: "2",
	key: "1w4ew1"
}], ["path", {
	d: "M7 11V7a5 5 0 0 1 10 0v4",
	key: "fwvmzm"
}]]), Te = Q("PlayIcon", [["polygon", {
	points: "6 3 20 12 6 21 6 3",
	key: "1oa8hb"
}]]), Ee = Q("RefreshCwIcon", [
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
]), De = Q("SearchIcon", [["circle", {
	cx: "11",
	cy: "11",
	r: "8",
	key: "4ej97u"
}], ["path", {
	d: "m21 21-4.3-4.3",
	key: "1qie3q"
}]]), Oe = Q("Share2Icon", [
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
]), ke = Q("SkipForwardIcon", [["polygon", {
	points: "5 4 15 12 5 20 5 4",
	key: "16p6eg"
}], ["line", {
	x1: "19",
	x2: "19",
	y1: "5",
	y2: "19",
	key: "futhcm"
}]]), Ae = Q("SquareIcon", [["rect", {
	width: "18",
	height: "18",
	x: "3",
	y: "3",
	rx: "2",
	key: "afitv7"
}]]), je = Q("WifiOffIcon", [
	["path", {
		d: "M12 20h.01",
		key: "zekei9"
	}],
	["path", {
		d: "M8.5 16.429a5 5 0 0 1 7 0",
		key: "1bycff"
	}],
	["path", {
		d: "M5 12.859a10 10 0 0 1 5.17-2.69",
		key: "1dl1wf"
	}],
	["path", {
		d: "M19 12.859a10 10 0 0 0-2.007-1.523",
		key: "4k23kn"
	}],
	["path", {
		d: "M2 8.82a15 15 0 0 1 4.177-2.643",
		key: "1grhjp"
	}],
	["path", {
		d: "M22 8.82a15 15 0 0 0-11.288-3.764",
		key: "z3jwby"
	}],
	["path", {
		d: "m2 2 20 20",
		key: "1ooewy"
	}]
]), Me = new Set(/* @__PURE__ */ "fn.let.mut.const.var.type.union.enum.tag.alias.spec.ext.static.shared.impl.node.if.else.for.break.continue.loop.is.in.on.as.to.return.next.view.move.copy.take.hold.true.false.nil.null.None.Some.Ok.Err.task.spawn.await.reply.go.use.pac.super.dep.has.and.or.routes.outlet.link.route.nav.grid".split(".")), Ne = new Set(/* @__PURE__ */ "int.uint.byte.i8.i16.i64.u8.u16.u64.usize.float.double.bool.char.void.str.String.cstr.Handle.linear.List.Map.Set.Option.Result.Link".split("."));
function Pe(e) {
	return e >= "0" && e <= "9";
}
function Fe(e) {
	return Pe(e) || e >= "a" && e <= "f" || e >= "A" && e <= "F";
}
function Ie(e) {
	return /[\p{L}_]/u.test(e);
}
function Le(e) {
	return /[\p{L}\p{N}_-]/u.test(e);
}
var Re = ee.define({
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
		return Pe(n) || n === "." && Pe(e.string[e.pos + 1] || "") || n === "0" && (e.string[e.pos + 1] === "x" || e.string[e.pos + 1] === "b") ? ze(e) : Ie(n) ? Be(e) : n === "#" ? (e.next(), e.match("if") || e.match("for") || e.match("is") ? "keyword" : e.match("[") ? "meta" : e.match("{") ? "macroName" : "operator") : n === "@" ? (e.next(), Ie(e.peek() || "") && e.eatWhile(Le), "attributeName") : e.match("==") || e.match("!=") || e.match("<=") || e.match(">=") || e.match("->") || e.match("=>") || e.match("..=") || e.match("??") || e.match("?.") || e.match(".?") || e.match("&&") || e.match("||") || e.match("+=") || e.match("-=") || e.match("*=") || e.match("/=") || e.match("%=") || n === "." && e.match("..") ? "operator" : n === "." && /[a-zA-Z]/.test(e.string[e.pos + 1] || "") ? (e.next(), e.eatWhile(/[a-zA-Z]/), "propertyName") : "+-*/%=<>!&|~:;,.[](){}".indexOf(n) >= 0 ? (e.next(), "operator") : (e.next(), null);
	},
	languageData: { commentTokens: {
		line: "//",
		block: {
			open: "/*",
			close: "*/"
		}
	} }
});
function ze(e) {
	let t = e.pos, n = e.peek();
	return n === "0" && (e.string[t + 1] === "x" || e.string[t + 1] === "X") ? (e.next(), e.next(), e.eatWhile(Fe), e.eatWhile(/[uUiIfFdD]/), "number") : n === "0" && (e.string[t + 1] === "b" || e.string[t + 1] === "B") ? (e.next(), e.next(), e.eatWhile(/[01]/), "number") : (e.eatWhile(Pe), e.eatWhile(/[_]/), e.eatWhile(Pe), e.peek() === "." && Pe(e.string[e.pos + 1] || "") && (e.next(), e.eatWhile(Pe), e.eatWhile(/[_]/), e.eatWhile(Pe)), (e.peek() === "e" || e.peek() === "E") && (e.next(), (e.peek() === "-" || e.peek() === "+") && e.next(), e.eatWhile(Pe)), e.match("usize") || e.match("i64") || e.match("i16") || e.match("i8") || e.match("u64") || e.match("u16") || e.match("u8") || e.match("u") || e.match("f") || e.match("d"), "number");
}
function Be(e) {
	e.eatWhile(Le);
	let t = e.current();
	return Me.has(t) ? "keyword" : Ne.has(t) ? "typeName" : e.string.slice(e.pos).trimStart()[0] === "(" ? "function" : "variableName";
}
//#endregion
//#region src/utils/floatingScroll.ts
var Ve = 24, He = 700, Ue = "floating-scroll-style";
function We() {
	try {
		return new URLSearchParams(window.location.search).has("fsbdebug");
	} catch {
		return !1;
	}
}
var Ge = 0;
function Ke(e, t) {
	let n = window;
	n.__fsbInstances ||= /* @__PURE__ */ new Map(), e === null ? n.__fsbInstances.delete(t) : n.__fsbInstances.set(e.id, e);
	let r = document.getElementById("fsb-hud");
	r && (r.innerHTML = Array.from(n.__fsbInstances.values()).map((e) => `[${e.id}] ${e.target} · scroll:${e.counters.scrollEvents} poll:${e.counters.polls} upd:${e.counters.updates} top:${e.counters.lastScrollTop}→${e.counters.lastThumbTop}`).join("<br>") || "(no floating-scrollbar instances)");
}
function qe() {
	if (!We() || document.getElementById("fsb-hud")) return;
	let e = document.createElement("div");
	e.id = "fsb-hud", e.style.cssText = "position:fixed;left:8px;bottom:8px;z-index:99999;background:#111c;color:#7ee787;font:11px/1.5 monospace;padding:8px 10px;border:1px solid #369;border-radius:4px;max-width:70vw;pointer-events:none;", document.body.appendChild(e), window.__fsbInstances && Ke(null);
}
function Je() {
	if (document.getElementById(Ue)) return;
	let e = document.createElement("style");
	e.id = Ue, e.textContent = "\n.fsb-rail {\n  position: absolute;\n  opacity: 0;\n  pointer-events: none;\n  transition: opacity 0.2s ease;\n  z-index: 8;\n}\n.fsb-rail.visible {\n  opacity: 1;\n  pointer-events: auto;\n}\n.fsb-y {\n  top: 3px;\n  right: 3px;\n  width: 12px;\n  height: calc(100% - 6px);\n}\n.fsb-x {\n  bottom: 3px;\n  left: 3px;\n  height: 12px;\n  width: calc(100% - 6px);\n}\n.fsb-thumb {\n  /* absolute inside the positioned rail so style.top/left actually move it */\n  position: absolute;\n  background: rgba(212, 212, 212, 0.18);\n  border-radius: 3px;\n  transition: background 0.15s ease;\n}\n.fsb-y .fsb-thumb {\n  left: 2.5px;\n  width: 7px;\n}\n.fsb-x .fsb-thumb {\n  top: 2.5px;\n  height: 7px;\n}\n.fsb-rail:hover .fsb-thumb,\n.fsb-thumb.dragging {\n  background: rgba(212, 212, 212, 0.35);\n}\n", document.head.appendChild(e);
}
function Ye(e, t) {
	Je(), qe();
	let n = ++Ge, r = We() ? {
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
	r && Ke(r);
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
		p || (r ? (r.counters.polls++, (u || T) && x(), Ke(r)) : (u || T) && x());
	}, 120);
	function h() {
		p || (x(), u = !0, _(), d && clearTimeout(d));
	}
	function g() {
		d && clearTimeout(d), d = setTimeout(() => {
			T || (u = !1, _());
		}, He);
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
			let n = Math.max(Ve, Math.round(e.clientHeight / e.scrollHeight * i)), a = Math.round(e.scrollTop / t * (i - n));
			s.style.height = n + "px", s.style.top = a + "px", r && (r.counters.updates++, r.counters.lastScrollTop = Math.round(e.scrollTop), r.counters.lastThumbTop = a);
		}
		if (b) {
			let t = Math.max(Ve, Math.round(e.clientWidth / e.scrollWidth * a)), r = Math.round(e.scrollLeft / n * (a - t));
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
		return parseFloat(s.style.height || String(Ve));
	}
	function O() {
		return parseFloat(l.style.width || String(Ve));
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
		p = !0, window.clearInterval(m), r && Ke(null, n), M.disconnect(), N.disconnect(), window.removeEventListener("resize", x), e.removeEventListener("scroll", C), i.removeEventListener("mouseenter", A), i.removeEventListener("mouseleave", j), o.remove(), c.remove(), a !== void 0 && (i.style.position = a), d && clearTimeout(d), cancelAnimationFrame(f);
	} };
}
//#endregion
//#region src/components/CodeEditor.vue?vue&type=script&setup=true&lang.ts
var Xe = /* @__PURE__ */ l({
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
					...G,
					K
				]),
				Re,
				q,
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
			}), s = Ye(o.scrollDOM, a.value);
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
}, Ze = /* @__PURE__ */ $(Xe, [["__scopeId", "data-v-b2c9dc90"]]), Qe = /* @__PURE__ */ $(/* @__PURE__ */ l({
	__name: "ScrollArea",
	setup(e, { expose: t }) {
		let n = y(null), r = y(null), o = null;
		return g(() => {
			!n.value || !r.value || (o = Ye(r.value, n.value));
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
}), [["__scopeId", "data-v-be12123c"]]), $e = (/* @__PURE__ */ ae((/* @__PURE__ */ X(((e, t) => {
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
	}, G = {
		scope: "title",
		begin: E,
		relevance: 0
	}, K = {
		begin: "\\.\\s*" + E,
		relevance: 0
	}, q = /* @__PURE__ */ Object.freeze({
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
		METHOD_GUARD: K,
		NUMBER_MODE: B,
		NUMBER_RE: D,
		PHRASAL_WORDS_MODE: F,
		QUOTE_STRING_MODE: P,
		REGEXP_MODE: U,
		RE_STARTERS_RE: A,
		SHEBANG: j,
		TITLE_MODE: W,
		UNDERSCORE_IDENT_RE: E,
		UNDERSCORE_TITLE_MODE: G
	});
	function ee(e, t) {
		e.input[e.index - 1] === "." && t.ignoreMatch();
	}
	function te(e, t) {
		e.className !== void 0 && (e.scope = e.className, delete e.className);
	}
	function ne(e, t) {
		t && e.beginKeywords && (e.begin = "\\b(" + e.beginKeywords.split(" ").join("|") + ")(?!\\.)(?=\\b|\\s)", e.__beforeBegin = ee, e.keywords = e.keywords || e.beginKeywords, delete e.beginKeywords, e.relevance === void 0 && (e.relevance = 0));
	}
	function re(e, t) {
		Array.isArray(e.illegal) && (e.illegal = y(...e.illegal));
	}
	function J(e, t) {
		if (e.match) {
			if (e.begin || e.end) throw Error("begin & end are not supported with match");
			e.begin = e.match, delete e.match;
		}
	}
	function ie(e, t) {
		e.relevance === void 0 && (e.relevance = 1);
	}
	var Y = (e, t) => {
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
	], Z = "keyword";
	function ae(e, t, n = Z) {
		let r = Object.create(null);
		return typeof e == "string" ? i(n, e.split(" ")) : Array.isArray(e) ? i(n, e) : Object.keys(e).forEach(function(n) {
			Object.assign(r, ae(e[n], t, n));
		}), r;
		function i(e, n) {
			t && (n = n.map((e) => e.toLowerCase())), n.forEach(function(t) {
				let n = t.split("|");
				r[n[0]] = [e, oe(n[0], n[1])];
			});
		}
	}
	function oe(e, t) {
		return t ? Number(t) : +!se(e);
	}
	function se(e) {
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
				te,
				J,
				ge,
				Y
			].forEach((e) => e(n, r)), e.compilerExtensions.forEach((e) => e(n, r)), n.__beforeBegin = null, [
				ne,
				re,
				ie
			].forEach((e) => e(n, r)), n.isCompiled = !0;
			let s = null;
			return typeof n.keywords == "object" && n.keywords.$pattern && (n.keywords = Object.assign({}, n.keywords), s = n.keywords.$pattern, delete n.keywords.$pattern), s ||= /\w+/, n.keywords &&= ae(n.keywords, e.case_insensitive), a.keywordPatternRe = t(s, !0), r && (n.begin ||= /\B|\b/, a.beginRe = t(a.begin), !n.end && !n.endsWithParent && (n.end = /\B|\b/), n.end && (a.endRe = t(a.end)), a.terminatorEnd = p(a.end) || "", n.endsWithParent && r.terminatorEnd && (a.terminatorEnd += (n.end ? "|" : "") + r.terminatorEnd)), n.illegal && (a.illegalRe = t(n.illegal)), n.contains ||= [], n.contains = [].concat(...n.contains.map(function(e) {
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
		for (let e in q) typeof q[e] == "object" && n(q[e]);
		return Object.assign(e, q), e;
	}, De = Ee({});
	De.newInstance = () => Ee({}), t.exports = De, De.HighlightJS = De, De.default = De;
})))())).default;
//#endregion
//#region node_modules/highlight.js/es/languages/rust.js
function et(e) {
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
function tt(e) {
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
var nt = "[A-Za-z$_][0-9A-Za-z$_]*", rt = /* @__PURE__ */ "as.in.of.if.for.while.finally.var.new.function.do.return.void.else.break.catch.instanceof.with.throw.case.default.try.switch.continue.typeof.delete.let.yield.const.class.debugger.async.await.static.import.from.export.extends.using".split("."), it = [
	"true",
	"false",
	"null",
	"undefined",
	"NaN",
	"Infinity"
], at = /* @__PURE__ */ "Object.Function.Boolean.Symbol.Math.Date.Number.BigInt.String.RegExp.Array.Float32Array.Float64Array.Int8Array.Uint8Array.Uint8ClampedArray.Int16Array.Int32Array.Uint16Array.Uint32Array.BigInt64Array.BigUint64Array.Set.Map.WeakSet.WeakMap.ArrayBuffer.SharedArrayBuffer.Atomics.DataView.JSON.Promise.Generator.GeneratorFunction.AsyncFunction.Reflect.Proxy.Intl.WebAssembly".split("."), ot = [
	"Error",
	"EvalError",
	"InternalError",
	"RangeError",
	"ReferenceError",
	"SyntaxError",
	"TypeError",
	"URIError"
], st = [
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
], ct = [
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
], lt = [].concat(st, at, ot);
function ut(e) {
	let t = e.regex, n = (e, { after: t }) => {
		let n = "</" + e[0].slice(1);
		return e.input.indexOf(n, t) !== -1;
	}, r = nt, i = {
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
		$pattern: nt,
		keyword: rt,
		literal: it,
		built_in: lt,
		"variable.language": ct
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
		keywords: { _: [...at, ...ot] }
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
			...st,
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
function dt(e) {
	let t = e.regex, n = ut(e), r = nt, i = [
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
		$pattern: nt,
		keyword: rt.concat([
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
		literal: it,
		built_in: lt.concat(i),
		"variable.language": ct
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
function ft(e) {
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
var pt = () => ({
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
}), mt = { class: "lines-container" }, ht = ["onClick"], gt = { class: "line-number" }, _t = ["innerHTML"], vt = {
	key: 0,
	class: "code-line"
}, yt = /* @__PURE__ */ $(/* @__PURE__ */ l({
	__name: "CodePreview",
	props: {
		code: {},
		language: {},
		highlightLines: {}
	},
	emits: ["line-click"],
	setup(o, { emit: s }) {
		$e.registerLanguage("rust", et), $e.registerLanguage("python", tt), $e.registerLanguage("typescript", dt), $e.registerLanguage("c", ft), $e.registerLanguage("abt", pt);
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
				return $e.highlight(c.code, { language: e }).value.split("\n");
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
		return (t, o) => (v(), n(Qe, { class: "code-preview" }, {
			default: k(() => [a("div", mt, [(v(!0), i(e, null, b(d.value, (e, t) => (v(), i("div", {
				key: t,
				class: p(["code-line", { highlighted: f(t + 1) }]),
				onClick: (e) => m(t + 1)
			}, [a("span", gt, C(t + 1), 1), a("span", {
				class: "line-content",
				innerHTML: e || " "
			}, null, 8, _t)], 10, ht))), 128)), d.value.length === 0 ? (v(), i("div", vt, [...o[0] ||= [a("span", { class: "line-number" }, "1", -1), a("span", { class: "line-content" }, null, -1)]])) : r("", !0)])]),
			_: 1
		}));
	}
}), [["__scopeId", "data-v-07e5fb27"]]), bt = {
	key: 0,
	class: "time-info"
}, xt = {
	key: 1,
	class: "stdout"
}, St = {
	key: 2,
	class: "stderr"
}, Ct = {
	key: 3,
	class: "result"
}, wt = {
	key: 4,
	class: "empty"
}, Tt = /* @__PURE__ */ $(/* @__PURE__ */ l({
	__name: "ConsoleOutput",
	props: {
		stdout: {},
		stderr: {},
		result: {},
		timeMs: {}
	},
	setup(e) {
		return (t, a) => (v(), n(Qe, { class: "console-output" }, {
			default: k(() => [
				e.timeMs > 0 ? (v(), i("div", bt, "Completed in " + C(e.timeMs) + "ms", 1)) : r("", !0),
				e.stdout ? (v(), i("pre", xt, C(e.stdout), 1)) : r("", !0),
				e.stderr ? (v(), i("pre", St, C(e.stderr), 1)) : r("", !0),
				e.result ? (v(), i("pre", Ct, "Result: " + C(e.result), 1)) : r("", !0),
				!e.stdout && !e.stderr && !e.result ? (v(), i("div", wt, "Click Run or press Ctrl+Enter to execute")) : r("", !0)
			]),
			_: 1
		}));
	}
}), [["__scopeId", "data-v-722a3db0"]]), Et = !1, Dt = null;
function Ot() {
	return Et ? Promise.resolve() : Dt || (Dt = new Promise((e, t) => {
		if (window.ts) {
			Et = !0, e();
			return;
		}
		let n = document.createElement("script");
		n.src = "https://cdn.jsdelivr.net/npm/typescript@5.7.3/lib/typescript.js", n.onload = () => {
			Et = !0, e();
		}, n.onerror = () => t(/* @__PURE__ */ Error("Failed to load TypeScript compiler")), document.head.appendChild(n);
	}), Dt);
}
async function kt(e) {
	try {
		await Ot();
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
var At = 500, jt = "// Welcome to Auto Playground!\nfn add(a int, b int) int {\n    a + b\n}\n\nlet result = add(3, 4)\nprint(result)";
function Mt(e = {}) {
	let n = e.apiBase ?? "/api", r = e.persistKey ?? "auto-playground:state", i = e.defaultSource ?? jt, a = e.preloadTargets ?? !0;
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
	let c = o(), l = y(c.source ?? i), u = y(""), d = y(""), f = y(""), p = y(0), m = y([]), h = y(!1), g = y(!1);
	async function _() {
		try {
			await fetch(`${n}/examples`, { method: "GET" }), g.value = !1;
		} catch {
			g.value = !0;
		}
		return !g.value;
	}
	let v = y(c.activeTab ?? "rust"), b = y(""), x = y(c.liveCompile ?? !0), S = y(c.projectDir), C = y([]);
	function w(e) {
		if (S.value && (e.project_dir = S.value), C.value.length > 0) {
			e.files = C.value;
			let t = C.value.find((e) => e.path === "main.at");
			t && (e.source = t.source);
		}
		return e;
	}
	let T = y({}), E = y(null), D = y([]), k = y([]), A = y({
		message: "",
		visible: !1
	}), j = null, M = t(() => {
		let e = b.value;
		if (!e) return "";
		let t = T.value[e];
		return t ? t.files.find((e) => e.path === t.selectedFile)?.code ?? t.files[0]?.code ?? "" : "";
	}), N = t(() => {
		let e = b.value;
		return e ? T.value[e]?.files ?? [] : [];
	}), P = t(() => {
		let e = b.value;
		return e ? T.value[e]?.selectedFile ?? "" : "";
	}), F = t(() => ""), I = t(() => {
		let e = b.value, t = /* @__PURE__ */ new Map();
		if (!e) return t;
		let n = T.value[e];
		if (!n) return t;
		for (let e of n.files) {
			let r = n.fileSourceMaps[e.path] ?? [];
			for (let n of r) {
				let r = n.source_file || F.value;
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
	}), L = t(() => {
		let e = b.value, t = /* @__PURE__ */ new Map();
		if (!e) return t;
		let n = T.value[e];
		if (!n) return t;
		for (let e of n.files) {
			let r = n.fileSourceMaps[e.path] ?? [];
			for (let n of r) {
				let r = n.source_file || F.value;
				t.has(e.path) || t.set(e.path, /* @__PURE__ */ new Map()), t.get(e.path).set(n.output_line, {
					sourceFile: r,
					sourceLine: n.source_line
				});
			}
		}
		return t;
	});
	function R() {
		E.value ? z(E.value) : (D.value = [], k.value = []);
	}
	function z(e) {
		E.value = e;
		let t = F.value, n = I.value.get(t)?.get(e) ?? [];
		k.value = n.map((e) => e.outputFile);
		let r = P.value;
		D.value = n.find((e) => e.outputFile === r)?.outputLines ?? [];
	}
	function B(e, t) {
		let n = L.value.get(e)?.get(t);
		if (!n) {
			H();
			return;
		}
		E.value = n.sourceLine;
		let r = I.value.get(n.sourceFile)?.get(n.sourceLine) ?? [];
		k.value = r.map((e) => e.outputFile), D.value = r.find((t) => t.outputFile === e)?.outputLines ?? [];
	}
	function V(e, t) {
		return L.value.get(e)?.get(t)?.sourceFile;
	}
	function H() {
		E.value = null, D.value = [], k.value = [];
	}
	async function U() {
		h.value = !0, u.value = "", d.value = "", f.value = "", m.value = [];
		try {
			let e = w({ source: l.value }), t = await (await fetch(`${n}/run`, {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify(e)
			})).json();
			g.value = !1, u.value = t.stdout || "", d.value = t.stderr || "", p.value = t.time_ms || 0, m.value = t.bytecode || [], t.result !== void 0 && t.result !== null && t.result !== "" && (f.value = t.result);
		} catch (e) {
			g.value = !0, d.value = `Network error: ${e.message}`;
		} finally {
			h.value = !1;
		}
	}
	async function W(e) {
		let t = T.value[e]?.files[0]?.code ?? "";
		if (!t.trim()) {
			d.value = `No ${e} code to run. Make sure the transpilation succeeded.`;
			return;
		}
		h.value = !0, u.value = "", d.value = "", f.value = "";
		try {
			if (e === "typescript") {
				let e = await kt(t);
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
				g.value = !1, u.value = r.stdout || "", d.value = r.stderr || "", p.value = r.time_ms || 0, r.result !== void 0 && r.result !== null && r.result !== "" && (f.value = r.result);
			}
		} catch (e) {
			g.value = !0, d.value = `Network error: ${e.message}`;
		} finally {
			h.value = !1;
		}
	}
	async function G(e) {
		h.value = !0;
		try {
			let t = w({
				source: l.value,
				target: e
			}), r = await (await fetch(`${n}/trans`, {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify(t)
			})).json();
			g.value = !1;
			let i = r.files ?? [], a = {};
			for (let e of i) a[e.path] = e.source_map ?? r.source_map ?? [];
			let o = i[0]?.path ?? "";
			b.value = e, T.value[e] = {
				files: i,
				fileSourceMaps: a,
				selectedFile: o
			}, R();
		} catch (t) {
			g.value = !0, b.value = e, T.value[e] = {
				files: [{
					path: "error.txt",
					code: `Error: ${t.message}`
				}],
				fileSourceMaps: { "error.txt": [] },
				selectedFile: "error.txt"
			}, R();
		} finally {
			h.value = !1;
		}
	}
	function K(e) {
		if (v.value = e, b.value = e, !T.value[e] && x.value) {
			G(e);
			return;
		}
		R();
	}
	function q(e, t) {
		let n = T.value[e];
		n && (n.selectedFile = t, R());
	}
	function ee(e) {
		l.value = e.source, S.value = e.project_dir, C.value = e.files ?? [], u.value = "", d.value = "", f.value = "", m.value = [], E.value = null, D.value = [], k.value = [];
	}
	function te() {
		if (typeof window > "u") return "";
		let e = JSON.stringify({
			source: l.value,
			activeTab: v.value,
			liveCompile: x.value,
			projectDir: S.value
		}), t = "#share=" + encodeURIComponent(btoa(e));
		return window.location.origin + window.location.pathname + t;
	}
	async function ne() {
		let e = te(), t = !1;
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
		A.value = {
			message: t ? "Share link copied to clipboard!" : "Failed to copy link",
			visible: !0
		}, setTimeout(() => {
			A.value.visible = !1;
		}, 2500);
	}
	O(l, () => {
		T.value = {}, x.value && (j && clearTimeout(j), j = setTimeout(() => {
			G(v.value);
		}, At));
	}), O([
		l,
		v,
		x,
		S
	], ([e, t, n, r]) => {
		s({
			source: e,
			activeTab: t,
			liveCompile: n,
			projectDir: r
		});
	}, { deep: !0 }), a && typeof window < "u" && setTimeout(() => {
		re();
	}, 100);
	async function re() {
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
					let t = w({
						source: l.value,
						target: e
					}), r = await (await fetch(`${n}/trans`, {
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
			for (let e of t) T.value[e.target] = {
				files: e.files,
				fileSourceMaps: e.fileSourceMaps,
				selectedFile: e.selectedFile
			};
			let r = v.value;
			T.value[r] && (b.value = r, R());
		} finally {
			h.value = !1;
		}
	}
	let J = y(null), ie = y(null), Y = y([]), X = y([]), Z = y(!1);
	async function ae() {
		h.value = !0, d.value = "";
		try {
			let e = await (await fetch(`${n}/agent-debug/start`, {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify({ source: l.value })
			})).json();
			J.value = e.session_id, Y.value = (e.bytecode || []).map((e) => ({
				offset: e.offset ?? e.idx ?? 0,
				mnemonic: e.mnemonic ?? e.op ?? "",
				operands: e.operands ?? e.args ?? "",
				line: e.line
			})), Z.value = !0, ie.value = null, X.value.length > 0 && await oe(X.value);
		} catch (e) {
			d.value = `Debug start error: ${e.message}`;
		} finally {
			h.value = !1;
		}
	}
	async function oe(e) {
		if (X.value = e, J.value) try {
			await fetch(`${n}/agent-debug/${J.value}/breakpoints`, {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify({ lines: e })
			});
		} catch {}
	}
	async function se(e) {
		if (J.value) {
			h.value = !0;
			try {
				let t = await (await fetch(`${n}/agent-debug/${J.value}/command`, {
					method: "POST",
					headers: { "Content-Type": "application/json" },
					body: JSON.stringify({ cmd: e })
				})).json();
				ie.value = t, t.stdout && (u.value = t.stdout), t.stderr && (d.value = t.stderr), t.result && (f.value = t.result), (t.status === "finished" || t.status === "error") && (Z.value = !1);
			} catch (e) {
				d.value = `Debug command error: ${e.message}`;
			} finally {
				h.value = !1;
			}
		}
	}
	async function ce() {
		if (J.value) {
			try {
				await fetch(`${n}/agent-debug/${J.value}`, { method: "DELETE" });
			} catch {}
			J.value = null, ie.value = null, Y.value = [], Z.value = !1;
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
		activeTab: v,
		transpiledCode: M,
		transpileTarget: b,
		liveCompile: x,
		projectDir: S,
		projectFiles: C,
		backendDown: g,
		retryBackend: _,
		transFiles: N,
		selectedTransFile: P,
		highlightedSourceLine: E,
		highlightedOutputLines: D,
		highlightedOutputFiles: k,
		shareToast: A,
		debugSessionId: J,
		debugState: ie,
		bytecode: Y,
		breakpoints: X,
		isDebugging: Z,
		run: U,
		runCode: W,
		transpile: G,
		switchTab: K,
		selectTransFile: q,
		loadExample: ee,
		highlightSourceLine: z,
		highlightOutputLine: B,
		getSourceFileForOutputLine: V,
		clearHighlight: H,
		share: ne,
		debugStart: ae,
		debugSetBreakpoints: oe,
		debugCommand: se,
		debugStop: ce
	};
}
//#endregion
//#region src/components/SnippetRunner.vue?vue&type=script&setup=true&lang.ts
var Nt = {
	key: 0,
	class: "snippet-actionbar"
}, Pt = ["title", "disabled"], Ft = {
	key: 0,
	class: "down-hint"
}, It = {
	key: 1,
	class: "target-hint"
}, Lt = {
	key: 1,
	class: "snippet-output"
}, Rt = {
	key: 0,
	class: "backend-down-card"
}, zt = { class: "bd-actions" }, Bt = /* @__PURE__ */ $(/* @__PURE__ */ l({
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
		let u = e, d = l, { source: f, stdout: h, stderr: _, resultCode: b, timeMs: S, isLoading: T, transpiledCode: E, liveCompile: D, transFiles: k, selectedTransFile: A, highlightedOutputLines: j, shareToast: M, debugState: N, bytecode: P, breakpoints: F, isDebugging: I, projectDir: L, projectFiles: R, backendDown: z, retryBackend: B, run: V, switchTab: H, selectTransFile: U, loadExample: W, share: G, highlightOutputLine: K, transpile: q, debugStart: ee, debugSetBreakpoints: te, debugCommand: ne, debugStop: re } = Mt({
			apiBase: u.apiBase || "/api",
			defaultSource: u.code,
			persistKey: !1,
			preloadTargets: !1
		}), J = y(!1), ie = {
			rust: "Rust",
			c: "C",
			python: "Python",
			typescript: "TypeScript",
			abt: "ABT"
		}, Y = t(() => u.target === "run" ? "" : ie[u.target]), X = t(() => u.target === "run" ? "Run (Ctrl+Enter)" : `Transpile to ${Y.value}`);
		O(z, (e) => {
			e && (J.value = !0);
		});
		async function Z() {
			await B() && (J.value = !1);
		}
		let ae = t(() => {
			if (u.fill || u.height !== "auto") return {};
			let e = f.value.split("\n").length;
			return { height: `${Math.min(480, Math.max(80, e * 20 + 16))}px` };
		}), oe = t(() => u.fill || u.height === "auto" ? {} : { height: u.height });
		async function se() {
			u.target === "run" ? await V() : await q(u.target), (h.value || _.value || b.value || E.value) && (J.value = !0);
		}
		function ce() {
			if (u.runHandler) {
				u.runHandler();
				return;
			}
			se();
		}
		return u.autorun && g(() => {
			ce();
		}), o({
			source: f,
			stdout: h,
			stderr: _,
			resultCode: b,
			timeMs: S,
			isLoading: T,
			transpiledCode: E,
			liveCompile: D,
			transFiles: k,
			selectedTransFile: A,
			highlightedOutputLines: j,
			shareToast: M,
			debugState: N,
			bytecode: P,
			breakpoints: F,
			isDebugging: I,
			projectDir: L,
			projectFiles: R,
			backendDown: z,
			retryBackend: B,
			run: V,
			switchTab: H,
			selectTransFile: U,
			loadExample: W,
			share: G,
			highlightOutputLine: K,
			transpile: q,
			debugStart: ee,
			debugSetBreakpoints: te,
			debugCommand: ne,
			debugStop: re,
			action: ce
		}), (t, o) => (v(), i("div", {
			class: p(["snippet-runner", {
				fill: e.fill,
				row: e.fill && e.orientation === "row",
				sized: !e.fill && e.height !== "auto"
			}]),
			style: m(oe.value)
		}, [
			e.actionBar ? (v(), i("div", Nt, [
				a("button", {
					class: p(["run-action", {
						busy: w(T),
						down: w(z)
					}]),
					title: w(z) ? "后端离线——见下方引导" : X.value,
					disabled: w(z),
					onClick: ce
				}, [w(z) ? (v(), n(w(je), {
					key: 0,
					size: 13
				})) : w(T) ? (v(), n(w(Ce), {
					key: 2,
					size: 13,
					class: "spin"
				})) : (v(), n(w(Te), {
					key: 1,
					size: 13
				}))], 10, Pt),
				w(z) ? (v(), i("span", Ft, "后端离线")) : e.target === "run" ? r("", !0) : (v(), i("span", It, "→ " + C(Y.value), 1)),
				o[3] ||= a("span", { class: "spacer" }, null, -1),
				a("button", {
					class: p(["output-toggle", { open: J.value }]),
					title: "Toggle output",
					onClick: o[0] ||= (e) => J.value = !J.value
				}, [c(w(me), { size: 13 })], 2)
			])) : r("", !0),
			a("div", {
				class: "snippet-editor",
				style: m(ae.value)
			}, [c(Ze, {
				"model-value": w(f),
				"onUpdate:modelValue": o[1] ||= (e) => f.value = e,
				"on-run": ce,
				"is-debugging": w(I),
				breakpoints: w(F),
				"current-debug-line": e.currentDebugLine ?? null,
				onBreakpointsChange: o[2] ||= (e) => d("breakpoints-change", e)
			}, null, 8, [
				"model-value",
				"is-debugging",
				"breakpoints",
				"current-debug-line"
			])], 4),
			J.value || e.fill ? (v(), i("div", Lt, [x(t.$slots, "output", {}, () => [w(z) ? (v(), i("div", Rt, [
				c(w(je), {
					size: 16,
					class: "bd-icon"
				}),
				o[6] ||= a("p", { class: "bd-title" }, "后端未启动——运行与转译暂不可用", -1),
				o[7] ||= a("p", { class: "bd-line" }, "笔记浏览与代码编辑不受影响。本地启动后端：", -1),
				o[8] ||= a("pre", { class: "bd-cmd" }, [a("code", null, "cargo run -p auto-playground")], -1),
				o[9] ||= a("p", { class: "bd-line" }, "或参考部署说明将后端与本站同域部署。", -1),
				a("div", zt, [a("button", {
					class: "bd-retry",
					onClick: Z
				}, [c(w(Ee), { size: 13 }), o[4] ||= s(" 重试连接 ", -1)]), o[5] ||= a("a", {
					class: "bd-link",
					href: "/playground#backend",
					title: "部署说明见 Playground 页底部"
				}, " 部署说明 ", -1)])
			])) : e.target === "run" ? (v(), n(Tt, {
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
			])) : (v(), n(yt, {
				key: 2,
				code: w(E),
				language: e.target
			}, null, 8, ["code", "language"]))], !0)])) : r("", !0)
		], 6));
	}
}), [["__scopeId", "data-v-8180c2ff"]]), Vt = ["data-offset", "onClick"], Ht = { class: "offset" }, Ut = { class: "mnemonic" }, Wt = { class: "operands" }, Gt = ["data-tip"], Kt = 320, qt = /* @__PURE__ */ $(/* @__PURE__ */ l({
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
			return t.length > Kt ? t.slice(0, Kt) + "…" : t;
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
		}), (o, c) => (v(), n(Qe, {
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
				a("span", Ht, C(E(n.offset)), 1),
				a("span", Ut, C(n.mnemonic), 1),
				a("span", Wt, [(v(!0), i(e, null, b(_(n), (t, n) => (v(), i(e, { key: n }, [t.tip ? (v(), i("span", {
					key: 0,
					class: "tok",
					"data-tip": t.tip
				}, C(t.text), 9, Gt)) : (v(), i(e, { key: 1 }, [s(C(t.text), 1)], 64))], 64))), 128))])
			], 10, Vt))), 128)), S.value.visible ? (v(), i("div", {
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
}), [["__scopeId", "data-v-96c74ca0"]]), Jt = { label: "Single-file" }, Yt = ["value"], Xt = { label: "Projects" }, Zt = ["value"], Qt = /* @__PURE__ */ $(/* @__PURE__ */ l({
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
			a("optgroup", Jt, [(v(!0), i(e, null, b(u.value, (e) => (v(), i("option", {
				key: e.name,
				value: JSON.stringify(e)
			}, C(e.name), 9, Yt))), 128))]),
			a("optgroup", Xt, [(v(!0), i(e, null, b(d.value, (e) => (v(), i("option", {
				key: e.name,
				value: JSON.stringify(e)
			}, C(e.name), 9, Zt))), 128))])
		], 544)), [[E, l.value]]);
	}
}), [["__scopeId", "data-v-578bba03"]]), $t = { class: "expected-output-panel" }, en = {
	key: 0,
	class: "eop-banner pending"
}, tn = {
	key: 1,
	class: "eop-banner match"
}, nn = {
	key: 2,
	class: "eop-banner diff"
}, rn = {
	key: 3,
	class: "eop-lines"
}, an = { class: "eop-ln" }, on = { class: "eop-text" }, sn = {
	key: 0,
	class: "eop-empty"
}, cn = {
	key: 4,
	class: "eop-lines"
}, ln = {
	key: 0,
	class: "eop-line same"
}, un = { class: "eop-ln" }, dn = { class: "eop-text" }, fn = {
	key: 0,
	class: "eop-line minus"
}, pn = { class: "eop-ln" }, mn = { class: "eop-text" }, hn = {
	key: 1,
	class: "eop-line plus"
}, gn = { class: "eop-ln" }, _n = { class: "eop-text" }, vn = /* @__PURE__ */ $(/* @__PURE__ */ l({
	__name: "ExpectedOutputPanel",
	props: {
		expected: {},
		actual: {},
		actualResult: {},
		expectedKind: {},
		hasRun: { type: Boolean }
	},
	setup(n) {
		let o = n;
		function s(e) {
			let t = e.replace(/\r\n/g, "\n").split("\n").map((e) => e.replace(/[ \t]+$/, ""));
			for (; t.length > 0 && t[t.length - 1] === "";) t.pop();
			return t;
		}
		let l = t(() => s(o.expected)), u = t(() => s(o.expectedKind === "result" ? o.actualResult ?? "" : o.actual)), d = t(() => o.hasRun ? f(l.value, u.value) ? "match" : "diff" : "pending");
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
		return (t, n) => (v(), i("div", $t, [d.value === "pending" ? (v(), i("div", en, [c(w(_e), { size: 13 }), n[0] ||= a("span", null, "期望输出（运行后在此对照实际结果）", -1)])) : d.value === "match" ? (v(), i("div", tn, [c(w(ge), { size: 13 }), a("span", null, "输出一致（" + C(l.value.length) + " 行）", 1)])) : (v(), i("div", nn, [c(w(ve), { size: 13 }), a("span", null, "输出有差异（" + C(m.value) + " / " + C(Math.max(l.value.length, u.value.length)) + " 行不一致）", 1)])), d.value === "pending" ? (v(), i("div", rn, [(v(!0), i(e, null, b(l.value, (e, t) => (v(), i("div", {
			key: t,
			class: "eop-line"
		}, [a("span", an, C(t + 1), 1), a("span", on, C(e), 1)]))), 128)), l.value.length === 0 ? (v(), i("div", sn, "（期望输出为空）")) : r("", !0)])) : (v(), i("div", cn, [(v(!0), i(e, null, b(p.value, (t) => (v(), i(e, { key: t }, [h(l.value, t - 1) === h(u.value, t - 1) ? (v(), i("div", ln, [
			a("span", un, C(t), 1),
			n[1] ||= a("span", { class: "eop-sign" }, "·", -1),
			a("span", dn, C(h(l.value, t - 1)), 1)
		])) : (v(), i(e, { key: 1 }, [t - 1 < l.value.length ? (v(), i("div", fn, [
			a("span", pn, C(t), 1),
			n[2] ||= a("span", { class: "eop-sign" }, "-", -1),
			a("span", mn, C(l.value[t - 1]), 1)
		])) : r("", !0), t - 1 < u.value.length ? (v(), i("div", hn, [
			a("span", gn, C(t), 1),
			n[3] ||= a("span", { class: "eop-sign" }, "+", -1),
			a("span", _n, C(u.value[t - 1]), 1)
		])) : r("", !0)], 64))], 64))), 128))]))]));
	}
}), [["__scopeId", "data-v-ea65cbec"]]), yn = ["onClick"], bn = { class: "file-name" }, xn = /* @__PURE__ */ $(/* @__PURE__ */ l({
	__name: "FileTree",
	props: {
		files: {},
		selected: {},
		mappedFiles: {}
	},
	emits: ["select"],
	setup(t) {
		return (r, o) => (v(), n(Qe, { class: "file-tree" }, {
			default: k(() => [(v(!0), i(e, null, b(t.files, (e) => (v(), i("div", {
				key: e.path,
				class: p(["file-item", {
					active: e.path === t.selected,
					mapped: t.mappedFiles?.includes(e.path)
				}]),
				onClick: (t) => r.$emit("select", e.path)
			}, [a("span", bn, C(e.path), 1)], 10, yn))), 128))]),
			_: 1
		}));
	}
}), [["__scopeId", "data-v-2da68091"]]), Sn = { class: "card-toolbar" }, Cn = { class: "toolbar-left" }, wn = ["title"], Tn = ["disabled"], En = { class: "toolbar-right" }, Dn = ["disabled"], On = ["disabled", "title"], kn = {
	key: 2,
	class: "debug-controls"
}, An = ["disabled"], jn = ["disabled"], Mn = ["disabled"], Nn = ["disabled"], Pn = ["disabled"], Fn = {
	key: 5,
	class: "switch-widget",
	title: "Toggle live transpile on edit"
}, In = { class: "switch" }, Ln = ["checked"], Rn = { class: "card-body" }, zn = {
	key: 0,
	class: "file-tabs"
}, Bn = ["title", "onClick"], Vn = { class: "card-output" }, Hn = { class: "output-tabs" }, Un = ["onClick"], Wn = ["title"], Gn = { class: "output-content" }, Kn = {
	key: 0,
	class: "card-backend-down"
}, qn = { class: "cbd-actions" }, Jn = {
	key: 4,
	class: "output-code-split"
}, Yn = {
	key: 0,
	class: "debug-panel"
}, Xn = {
	key: 0,
	class: "debug-section"
}, Zn = { class: "debug-section-title" }, Qn = { class: "debug-stack" }, $n = {
	key: 1,
	class: "debug-section"
}, er = { class: "frame-name" }, tr = { class: "frame-info" }, nr = {
	key: 2,
	class: "debug-section"
}, rr = { class: "debug-locals" }, ir = { class: "local-idx" }, ar = {
	key: 3,
	class: "debug-registers"
}, or = "fn main() {\n    let message = \"Hello from Auto!\"\n    print(message)\n}", sr = "main.at", cr = /* @__PURE__ */ $(/* @__PURE__ */ l({
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
		expectedOutput: { default: null },
		expectedKind: { default: "stdout" },
		files: { default: null },
		projectDir: { default: null },
		ideMode: {
			type: [Boolean, null],
			default: null
		},
		toolbar: { default: () => ({}) },
		exampleSelector: {
			type: Boolean,
			default: !1
		}
	},
	emits: ["ide-mode"],
	setup(l, { expose: u, emit: d }) {
		let f = l, h = d, _ = t(() => f.code ?? or), x = t(() => ({
			transpile: f.toolbar?.transpile ?? !0,
			share: f.toolbar?.share ?? !0,
			debug: f.toolbar?.debug ?? !0,
			live: f.toolbar?.live ?? !0
		})), S = y(null), T = t(() => S.value?.isLoading ?? !1), D = t(() => S.value?.isDebugging ?? !1), j = t(() => S.value?.debugState ?? null), M = t(() => S.value?.stdout ?? ""), N = t(() => S.value?.stderr ?? ""), P = t(() => S.value?.resultCode ?? ""), F = t(() => S.value?.timeMs ?? 0), I = t(() => S.value?.bytecode ?? []), L = t(() => S.value?.transpiledCode ?? ""), R = t(() => S.value?.transFiles ?? []), z = t(() => S.value?.selectedTransFile ?? ""), B = t(() => S.value?.highlightedOutputLines ?? []), V = t(() => S.value?.liveCompile ?? !1), H = t(() => S.value?.breakpoints ?? []), U = t(() => S.value?.shareToast), W = t(() => S.value?.backendDown ?? !1), G = y("Output"), K = y(f.target), q = y(!1), ee = y(!1), te = t(() => {
			let e = [
				"Output",
				"rust",
				"c",
				"python",
				"typescript",
				"abt",
				"Bytecode"
			];
			return f.expectedOutput ? ["Expected", ...e] : e;
		}), ne = {
			Expected: "期望输出",
			Output: "Output",
			rust: "Rust",
			c: "C",
			python: "Python",
			typescript: "TS",
			abt: "ABT",
			Bytecode: "Bytecode"
		}, re = t(() => f.height === "auto" ? { minHeight: "480px" } : { height: f.height }), J = t(() => G.value !== "Output" && G.value !== "Bytecode" && !!L.value), ie = t(() => {
			let e = G.value;
			return e !== "Output" && e !== "Bytecode" && R.value.length > 1;
		});
		async function Y() {
			let e = S.value;
			e && (ce(e), K.value === "run" ? (await e.run(), ee.value = !0, G.value = f.expectedOutput ? "Expected" : "Output") : (e.switchTab(K.value), G.value = K.value));
		}
		let X = y(sr), Z = y(/* @__PURE__ */ new Map()), ae = t(() => (f.files ?? []).map((e) => ({ path: e.path })));
		function oe() {
			let e = S.value;
			e && Z.value.set(X.value, e.source);
		}
		function se(e) {
			if (e === X.value) return;
			oe(), X.value = e;
			let t = S.value;
			t && (t.source = Z.value.get(e) ?? "");
		}
		function ce(e) {
			!f.files || f.files.length === 0 || (oe(), e.projectFiles = [...Z.value.entries()].map(([e, t]) => ({
				path: e,
				source: t
			})), e.projectDir = f.projectDir ?? void 0);
		}
		g(() => {
			if (!f.files || f.files.length === 0) return;
			let e = /* @__PURE__ */ new Map();
			for (let t of f.files) e.set(t.path, t.content);
			Z.value = e, X.value = e.has(sr) ? sr : f.files[0].path;
		}), O(K, (e) => {
			e !== "run" && V.value && (S.value?.switchTab(e), G.value = e);
		});
		function Q(e) {
			G.value = e, e !== "Output" && e !== "Bytecode" && e !== "Expected" ? (K.value = e, S.value?.switchTab(e)) : e === "Output" && (K.value = "run");
		}
		function me(e) {
			S.value?.selectTransFile(G.value, e);
		}
		function he(e) {
			S.value?.loadExample(e), G.value = "Output", K.value = "run";
		}
		function ge(e) {
			S.value?.highlightOutputLine(z.value, e);
		}
		function _e(e) {
			let t = S.value;
			t && (t.liveCompile = e.target.checked);
		}
		async function ve() {
			await S.value?.share();
		}
		async function xe() {
			if (L.value) try {
				await navigator.clipboard.writeText(L.value), q.value = !0, setTimeout(() => {
					q.value = !1;
				}, 2e3);
			} catch {}
		}
		async function Se() {
			await S.value?.debugStart(), G.value = "Bytecode";
		}
		async function De() {
			await S.value?.debugStop(), G.value = "Output";
		}
		function Me(e) {
			S.value?.debugCommand(e);
		}
		function Ne(e) {
			S.value?.debugSetBreakpoints(e);
		}
		function Pe(e) {}
		async function Fe() {
			await S.value?.retryBackend();
		}
		return u({ run: Y }), (t, u) => (v(), i(e, null, [a("div", {
			class: "playground-card",
			style: m(re.value)
		}, [a("div", Sn, [a("div", Cn, [
			c(w(ye), { size: 16 }),
			u[8] ||= a("span", { class: "toolbar-title" }, "Auto Playground", -1),
			l.exampleSelector ? (v(), n(Qt, {
				key: 0,
				"api-base": l.apiBase || "/api",
				onSelect: he
			}, null, 8, ["api-base"])) : r("", !0),
			l.ideMode === null ? r("", !0) : (v(), i("span", {
				key: 1,
				class: "ide-entry",
				title: l.ideMode ? "切换到全功能 IDE（文件树/调试/回放）" : "IDE 模式仅在后端自服务页可用——本地运行 cargo run -p auto-playground 后访问"
			}, [a("button", {
				class: "ide-btn",
				disabled: !l.ideMode,
				onClick: u[0] ||= (e) => h("ide-mode")
			}, [c(w(le), { size: 14 }), u[7] ||= s(" 在 IDE 中打开 ", -1)], 8, Tn)], 8, wn))
		]), a("div", En, [
			x.value.transpile ? A((v(), i("select", {
				key: 0,
				"onUpdate:modelValue": u[1] ||= (e) => K.value = e,
				class: "target-select",
				disabled: !!D.value
			}, [...u[9] ||= [o("<option value=\"run\" data-v-6b69b693>Run</option><option value=\"rust\" data-v-6b69b693>→ Rust</option><option value=\"c\" data-v-6b69b693>→ C</option><option value=\"python\" data-v-6b69b693>→ Python</option><option value=\"typescript\" data-v-6b69b693>→ TypeScript</option><option value=\"abt\" data-v-6b69b693>→ ABT</option>", 6)]], 8, Dn)), [[E, K.value]]) : r("", !0),
			D.value ? x.value.debug ? (v(), i("div", kn, [
				a("button", {
					class: "debug-btn continue",
					onClick: u[2] ||= (e) => Me("continue"),
					disabled: T.value,
					title: "Continue"
				}, [c(w(Te), { size: 14 })], 8, An),
				a("button", {
					class: "debug-btn step",
					onClick: u[3] ||= (e) => Me("step"),
					disabled: T.value,
					title: "Step Into"
				}, [c(w(ue), { size: 14 })], 8, jn),
				a("button", {
					class: "debug-btn step-over",
					onClick: u[4] ||= (e) => Me("step_over"),
					disabled: T.value,
					title: "Step Over"
				}, [c(w(ke), { size: 14 })], 8, Mn),
				a("button", {
					class: "debug-btn step-out",
					onClick: u[5] ||= (e) => Me("step_out"),
					disabled: T.value,
					title: "Step Out"
				}, [c(w(de), { size: 14 })], 8, Nn)
			])) : r("", !0) : (v(), i("button", {
				key: 1,
				class: p(["run-btn", { down: W.value }]),
				onClick: Y,
				disabled: T.value || W.value,
				title: W.value ? "后端离线——本地启动：cargo run -p auto-playground" : void 0
			}, [W.value ? (v(), n(w(je), {
				key: 0,
				size: 14
			})) : T.value ? (v(), n(w(Ce), {
				key: 2,
				size: 14,
				class: "spin"
			})) : (v(), n(w(Te), {
				key: 1,
				size: 14
			})), s(" " + C(W.value ? "后端离线" : T.value ? "Running..." : "Run"), 1)], 10, On)),
			x.value.debug && D.value ? (v(), i("button", {
				key: 3,
				class: "stop-btn",
				onClick: De,
				title: "Stop Debug"
			}, [c(w(Ae), { size: 14 }), u[10] ||= s(" Stop ", -1)])) : x.value.debug ? (v(), i("button", {
				key: 4,
				class: "debug-start-btn",
				onClick: Se,
				disabled: T.value,
				title: "Start Debug"
			}, [c(w(fe), { size: 14 }), u[11] ||= s(" Debug ", -1)], 8, Pn)) : r("", !0),
			x.value.live && !D.value ? (v(), i("label", Fn, [u[13] ||= a("span", { class: "switch-label" }, "Live", -1), a("span", In, [a("input", {
				type: "checkbox",
				checked: !!V.value,
				onChange: u[6] ||= (e) => _e(e)
			}, null, 40, Ln), u[12] ||= a("span", { class: "slider" }, null, -1)])])) : r("", !0),
			x.value.share ? (v(), i("button", {
				key: 6,
				class: "icon-btn share-btn",
				onClick: ve,
				title: "Copy shareable link"
			}, [c(w(Oe), { size: 14 })])) : r("", !0)
		])]), a("div", Rn, [ae.value.length > 1 ? (v(), i("div", zn, [(v(!0), i(e, null, b(ae.value, (e) => (v(), i("button", {
			key: e.path,
			class: p(["file-tab", {
				active: e.path === X.value,
				entry: e.path === sr
			}]),
			title: e.path === sr ? "入口文件（entry 锁定）" : e.path,
			onClick: (t) => se(e.path)
		}, [e.path === sr ? (v(), n(w(we), {
			key: 0,
			size: 10
		})) : r("", !0), a("span", null, C(e.path), 1)], 10, Bn))), 128))])) : r("", !0), c(Bt, {
			ref_key: "runner",
			ref: S,
			code: _.value,
			"api-base": l.apiBase,
			autorun: l.autorun,
			height: l.height,
			"action-bar": !1,
			fill: "",
			orientation: "row",
			"is-debugging": !!D.value,
			breakpoints: H.value ?? [],
			"current-debug-line": j.value?.line ?? null,
			"run-handler": Y,
			onBreakpointsChange: Ne
		}, {
			output: k(() => [a("div", Vn, [
				a("div", Hn, [
					(v(!0), i(e, null, b(te.value, (e) => (v(), i("button", {
						key: e,
						class: p(["tab-btn", { active: G.value === e }]),
						onClick: (t) => Q(e)
					}, C(ne[e]), 11, Un))), 128)),
					u[14] ||= a("div", { class: "spacer" }, null, -1),
					D.value && j.value ? (v(), i("span", {
						key: 0,
						class: p(["debug-status", j.value.status])
					}, C(j.value.status), 3)) : r("", !0),
					J.value ? (v(), i("button", {
						key: 1,
						class: "icon-btn copy-btn",
						onClick: xe,
						title: q.value ? "Copied!" : "Copy code"
					}, [q.value ? (v(), n(w(pe), {
						key: 1,
						size: 14
					})) : (v(), n(w(be), {
						key: 0,
						size: 14
					}))], 8, Wn)) : r("", !0)
				]),
				a("div", Gn, [W.value ? (v(), i("div", Kn, [
					c(w(je), {
						size: 16,
						class: "cbd-icon"
					}),
					u[16] ||= a("p", { class: "cbd-title" }, "后端未启动——运行与转译暂不可用", -1),
					u[17] ||= a("p", { class: "cbd-line" }, "笔记浏览与代码编辑不受影响。本地启动后端：", -1),
					u[18] ||= a("pre", { class: "cbd-cmd" }, [a("code", null, "cargo run -p auto-playground")], -1),
					u[19] ||= a("p", { class: "cbd-line" }, [
						s(" 或参考 "),
						a("a", {
							class: "cbd-link",
							href: "/playground#backend"
						}, "部署说明"),
						s(" 将后端与本站同域部署。 ")
					], -1),
					a("div", qn, [a("button", {
						class: "cbd-retry",
						onClick: Fe
					}, [c(w(Ee), { size: 13 }), u[15] ||= s(" 重试连接 ", -1)])])
				])) : G.value === "Expected" ? (v(), n(vn, {
					key: 1,
					expected: l.expectedOutput ?? "",
					actual: M.value ?? "",
					"actual-result": P.value ?? "",
					"expected-kind": l.expectedKind,
					"has-run": ee.value
				}, null, 8, [
					"expected",
					"actual",
					"actual-result",
					"expected-kind",
					"has-run"
				])) : G.value === "Output" ? (v(), n(Tt, {
					key: 2,
					stdout: M.value ?? "",
					stderr: N.value ?? "",
					result: P.value ?? "",
					"time-ms": F.value ?? 0
				}, null, 8, [
					"stdout",
					"stderr",
					"result",
					"time-ms"
				])) : G.value === "Bytecode" ? (v(), n(qt, {
					key: 3,
					bytecode: I.value ?? [],
					"current-ip": j.value?.ip,
					onOffsetClick: Pe
				}, null, 8, ["bytecode", "current-ip"])) : (v(), i("div", Jn, [ie.value ? (v(), n(xn, {
					key: 0,
					files: R.value ?? [],
					selected: z.value ?? "",
					onSelect: me
				}, null, 8, ["files", "selected"])) : r("", !0), c(yt, {
					code: L.value ?? "",
					language: G.value,
					"highlight-lines": B.value ?? [],
					onLineClick: ge
				}, null, 8, [
					"code",
					"language",
					"highlight-lines"
				])]))]),
				D.value && j.value ? (v(), i("div", Yn, [
					j.value.stack.length ? (v(), i("div", Xn, [a("div", Zn, "Stack (" + C(j.value.stack.length) + ")", 1), a("div", Qn, [(v(!0), i(e, null, b(j.value.stack.slice(-8), (e, t) => (v(), i("span", {
						key: t,
						class: "stack-item"
					}, C(e), 1))), 128))])])) : r("", !0),
					j.value.call_stack.length ? (v(), i("div", $n, [u[20] ||= a("div", { class: "debug-section-title" }, "Call Stack", -1), (v(!0), i(e, null, b(j.value.call_stack, (e, t) => (v(), i("div", {
						key: t,
						class: "call-frame"
					}, [a("span", er, C(e.fn_name || "<root>"), 1), a("span", tr, "line " + C(e.line) + ", bp=" + C(e.bp), 1)]))), 128))])) : r("", !0),
					j.value.locals.length ? (v(), i("div", nr, [u[21] ||= a("div", { class: "debug-section-title" }, "Locals", -1), a("div", rr, [(v(!0), i(e, null, b(j.value.locals, (e, t) => (v(), i("span", {
						key: t,
						class: "local-item"
					}, [a("span", ir, "[" + C(e.index) + "]", 1), s(" " + C(e.value), 1)]))), 128))])])) : r("", !0),
					j.value.registers ? (v(), i("div", ar, " IP=" + C(j.value.registers.ip) + " BP=" + C(j.value.registers.bp) + " SP=" + C(j.value.registers.sp), 1)) : r("", !0)
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
		])])], 4), a("div", { class: p(["toast", { visible: U.value?.visible }]) }, C(U.value?.message), 3)], 64));
	}
}), [["__scopeId", "data-v-6b69b693"]]), lr = /* @__PURE__ */ l({
	__name: "AutoPlayground",
	props: {
		code: { default: "fn main() {\n    let message = \"Hello from Auto!\"\n    print(message)\n}" },
		apiUrl: { default: "" },
		height: { default: "500px" }
	},
	setup(e) {
		let t = e, r = t.apiUrl ? `${t.apiUrl}/api` : "";
		return (t, i) => (v(), n(cr, {
			code: e.code,
			"api-base": w(r),
			height: e.height
		}, null, 8, [
			"code",
			"api-base",
			"height"
		]));
	}
}), ur = { class: "debug-toolbar" }, dr = [
	"disabled",
	"onClick",
	"title"
], fr = { class: "icon" }, pr = { class: "label" }, mr = ["title"], hr = { class: "icon" }, gr = { class: "label" }, _r = ["disabled"], vr = /* @__PURE__ */ $(/* @__PURE__ */ l({
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
		return (r, o) => (v(), i("div", ur, [
			(v(), i(e, null, b(n, (e) => a("button", {
				key: e.cmd,
				disabled: !t.isPaused,
				onClick: (t) => r.$emit("command", e.cmd),
				title: e.title
			}, [a("span", fr, C(e.icon), 1), a("span", pr, C(e.label), 1)], 8, dr)), 64)),
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
			}, [a("span", hr, C(t.isRecording ? "⏹" : "⏺"), 1), a("span", gr, C(t.isRecording ? "Recording" : "Record"), 1)], 10, mr),
			a("button", {
				class: "save-btn",
				onClick: o[2] ||= (e) => r.$emit("exportRecording"),
				disabled: !t.hasRecording,
				title: "Export Replay File"
			}, [...o[4] ||= [a("span", { class: "icon" }, "💾", -1), a("span", { class: "label" }, "Save", -1)]], 8, _r)
		]));
	}
}), [["__scopeId", "data-v-8068f5da"]]), yr = { class: "replay-toolbar" }, br = ["title"], xr = { class: "icon" }, Sr = { class: "timeline" }, Cr = ["max", "value"], wr = { class: "frame-info" }, Tr = /* @__PURE__ */ $(/* @__PURE__ */ l({
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
		return (t, n) => (v(), i("div", yr, [
			a("button", {
				onClick: n[0] ||= (n) => e.isPlaying ? t.$emit("pause") : t.$emit("play"),
				title: e.isPlaying ? "Pause" : "Play"
			}, [a("span", xr, C(e.isPlaying ? "⏸" : "▶"), 1)], 8, br),
			a("button", {
				onClick: n[1] ||= (e) => t.$emit("stepBackward"),
				title: "Step Backward (←)"
			}, [...n[3] ||= [a("span", { class: "icon" }, "⏮", -1)]]),
			a("button", {
				onClick: n[2] ||= (e) => t.$emit("stepForward"),
				title: "Step Forward (→)"
			}, [...n[4] ||= [a("span", { class: "icon" }, "⏭", -1)]]),
			a("div", Sr, [a("input", {
				type: "range",
				min: 0,
				max: Math.max(0, s.value - 1),
				value: o.value,
				onInput: l,
				class: "timeline-slider"
			}, null, 40, Cr), a("span", wr, "Frame " + C(o.value + 1) + " / " + C(s.value), 1)]),
			n[5] ||= a("div", { class: "replay-badge" }, "🔁 Replay Mode", -1)
		]));
	}
}), [["__scopeId", "data-v-1b1084c5"]]), Er = { class: "aux-section" }, Dr = {
	key: 0,
	class: "var-group"
}, Or = { class: "var-name" }, kr = { class: "var-value" }, Ar = {
	key: 1,
	class: "var-group"
}, jr = { class: "var-name" }, Mr = { class: "var-value" }, Nr = {
	key: 2,
	class: "var-group"
}, Pr = { class: "var-name" }, Fr = { class: "var-value" }, Ir = {
	key: 3,
	class: "empty"
}, Lr = { class: "aux-section" }, Rr = { class: "callstack-list" }, zr = { class: "cs-name" }, Br = { class: "cs-line" }, Vr = {
	key: 0,
	class: "empty"
}, Hr = { class: "aux-section compact" }, Ur = { class: "reg-row" }, Wr = { class: "reg-value" }, Gr = { class: "reg-row" }, Kr = { class: "reg-value" }, qr = { class: "reg-row" }, Jr = { class: "reg-value" }, Yr = {
	key: 0,
	class: "aux-section stdout-section"
}, Xr = { class: "stdout-text" }, Zr = /* @__PURE__ */ $(/* @__PURE__ */ l({
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
		return (t, g) => (v(), n(Qe, { class: "debug-aux-panel" }, {
			default: k(() => [
				a("div", Er, [
					g[3] ||= a("div", { class: "aux-title" }, "Variables", -1),
					s.state?.args?.length ? (v(), i("div", Dr, [g[0] ||= a("div", { class: "var-group-title" }, "Arguments", -1), (v(!0), i(e, null, b(s.state.args, (e) => (v(), i("div", {
						key: "arg" + e.index,
						class: "var-row"
					}, [a("span", Or, "arg" + C(e.index), 1), a("span", kr, C(e.value), 1)]))), 128))])) : r("", !0),
					s.state?.locals?.length ? (v(), i("div", Ar, [g[1] ||= a("div", { class: "var-group-title" }, "Locals", -1), (v(!0), i(e, null, b(s.state.locals, (e) => (v(), i("div", {
						key: "loc" + e.index,
						class: "var-row"
					}, [a("span", jr, "local" + C(e.index), 1), a("span", Mr, C(e.value), 1)]))), 128))])) : r("", !0),
					d.value.length ? (v(), i("div", Nr, [g[2] ||= a("div", { class: "var-group-title" }, "Stack Top", -1), (v(!0), i(e, null, b(d.value, (e, t) => (v(), i("div", {
						key: "stk" + t,
						class: p(["var-row", {
							"is-top": t === 0,
							"is-pushed": l.value && t === 0,
							"is-popped": u.value && t === 0
						}])
					}, [a("span", Pr, "[" + C(e.distFromTop) + "]", 1), a("span", Fr, C(e.value), 1)], 2))), 128))])) : r("", !0),
					m.value ? r("", !0) : (v(), i("div", Ir, "No variables"))
				]),
				a("div", Lr, [
					g[4] ||= a("div", { class: "aux-title" }, "Call Stack", -1),
					a("div", Rr, [(v(!0), i(e, null, b(f.value, (e, t) => (v(), i("div", {
						key: t,
						class: p(["callstack-item", { "is-current": t === 0 }])
					}, [a("span", zr, C(e.fn_name ?? "<main>"), 1), a("span", Br, ":" + C(e.line), 1)], 2))), 128))]),
					s.state?.call_stack?.length ? r("", !0) : (v(), i("div", Vr, "No frames"))
				]),
				a("div", Hr, [
					g[8] ||= a("div", { class: "aux-title" }, "Registers", -1),
					a("div", Ur, [g[5] ||= a("span", { class: "reg-label" }, "IP", -1), a("span", Wr, C(h(o.state?.registers.ip)), 1)]),
					a("div", Gr, [g[6] ||= a("span", { class: "reg-label" }, "BP", -1), a("span", Kr, C(h(o.state?.registers.bp)), 1)]),
					a("div", qr, [g[7] ||= a("span", { class: "reg-label" }, "SP", -1), a("span", Jr, C(h(o.state?.registers.sp)), 1)])
				]),
				o.state?.stdout ? (v(), i("div", Yr, [g[9] ||= a("div", { class: "aux-title" }, "Output", -1), c(Qe, { class: "stdout-scroll" }, {
					default: k(() => [a("pre", Xr, C(o.state.stdout), 1)]),
					_: 1
				})])) : r("", !0)
			]),
			_: 1
		}));
	}
}), [["__scopeId", "data-v-ee14fb49"]]), Qr = { class: "playground" }, $r = { class: "toolbar" }, ei = { class: "toolbar-left" }, ti = ["title"], ni = ["href", "title"], ri = {
	key: 1,
	class: "title"
}, ii = { class: "toolbar-right" }, ai = ["disabled", "title"], oi = ["disabled"], si = ["disabled"], ci = ["disabled"], li = { class: "trans-current" }, ui = { class: "workspace" }, di = { class: "main-row" }, fi = { class: "pane-header" }, pi = { key: 0 }, mi = { key: 1 }, hi = {
	key: 0,
	class: "active-file-name"
}, gi = { class: "pane-body" }, _i = {
	key: 0,
	class: "preview-pane"
}, vi = { class: "pane-header" }, yi = ["disabled"], bi = {
	key: 0,
	class: "output-pane"
}, xi = { class: "pane-header" }, Si = { class: "output-body" }, Ci = /* @__PURE__ */ $(/* @__PURE__ */ l({
	__name: "PlaygroundLayout",
	props: {
		source: {},
		isLoading: { type: Boolean },
		mode: {},
		noteMeta: {},
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
		return (t, u) => (v(), i("div", Qr, [
			a("header", $r, [a("div", ei, [l.noteMeta ? (v(), i(e, { key: 0 }, [
				a("h1", {
					class: "title",
					title: l.noteMeta.sourcePath
				}, C(l.noteMeta.title), 9, ti),
				a("span", { class: p(["note-badge", `t-${l.noteMeta.sourceType}`]) }, C(l.noteMeta.sourceType), 3),
				a("a", {
					class: "note-source-chip",
					href: `${(l.noteMeta.repoBase ?? "https://github.com/auto-stack/auto-lang").replace(/\/$/, "")}/blob/master/${l.noteMeta.sourcePath}`,
					target: "_blank",
					rel: "noopener",
					title: l.noteMeta.sourcePath
				}, C(l.noteMeta.sourcePath), 9, ni)
			], 64)) : (v(), i("h1", ri, "Auto Playground")), c(Qt, { onSelect: B })]), a("div", ii, [
				!l.isDebugging && !l.isReplayMode ? (v(), i("button", {
					key: 0,
					class: "toolbar-btn load-replay-btn",
					onClick: u[0] ||= (e) => t.$emit("loadReplay"),
					title: "Load Replay File"
				}, [...u[21] ||= [a("span", { class: "icon" }, "📂", -1), a("span", { class: "label" }, "Load Replay", -1)]])) : r("", !0),
				a("button", {
					class: "toolbar-btn share-btn",
					onClick: u[1] ||= (e) => t.$emit("share"),
					title: "Copy shareable link"
				}, [...u[22] ||= [a("svg", {
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
				}, [u[23] ||= o("<svg width=\"14\" height=\"14\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2.5\" stroke-linecap=\"round\" stroke-linejoin=\"round\" data-v-6b1ae17a><path d=\"M12 2a10 10 0 0 1 10 10\" data-v-6b1ae17a></path><path d=\"M12 2a10 10 0 0 0-10 10\" data-v-6b1ae17a></path><path d=\"M12 12l4-4\" data-v-6b1ae17a></path><path d=\"M12 12l-4-4\" data-v-6b1ae17a></path><path d=\"M12 12l4 4\" data-v-6b1ae17a></path><path d=\"M12 12l-4 4\" data-v-6b1ae17a></path></svg>", 1), s(" " + C(l.isDebugging ? "Exit Debug" : "Debug"), 1)], 10, ai),
				l.isDebugging ? r("", !0) : (v(), i("button", {
					key: 1,
					class: "toolbar-btn run-btn",
					onClick: u[3] ||= (...e) => d.onRun && d.onRun(...e),
					disabled: l.isLoading || l.isReplayMode
				}, C(l.isLoading ? "Running..." : "Run (Ctrl+Enter)"), 9, oi)),
				l.isDebugging ? r("", !0) : (v(), i("div", {
					key: 2,
					class: p(["trans-split-btn", { disabled: l.isLoading || l.isReplayMode }]),
					title: "Transpile to target language"
				}, [a("button", {
					class: "trans-main",
					onClick: u[4] ||= (...e) => d.onTrans && d.onTrans(...e),
					disabled: l.isLoading || l.isReplayMode
				}, " Trans ", 8, si), a("div", {
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
					}, [...u[24] ||= [o("<option value=\"rust\" data-v-6b1ae17a>Rust</option><option value=\"c\" data-v-6b1ae17a>C</option><option value=\"python\" data-v-6b1ae17a>Python</option><option value=\"typescript\" data-v-6b1ae17a>TypeScript</option><option value=\"abt\" data-v-6b1ae17a>ABT</option>", 5)]], 40, ci), [[E, h.value]]),
					a("span", li, C(_.value), 1),
					u[25] ||= a("span", { class: "trans-arrow" }, [a("svg", {
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
			l.isDebugging || l.hasRecording ? (v(), n(vr, {
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
			l.isReplayMode ? (v(), n(Tr, {
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
			a("div", ui, [a("div", di, [a("div", { class: p(["editor-pane", { "with-preview": l.mode !== "editor" }]) }, [a("div", fi, [l.isReplayMode ? (v(), i("span", pi, "Replay")) : (v(), i("span", mi, [u[26] ||= s("Auto ", -1), l.activeFile ? (v(), i("span", hi, "· " + C(l.activeFile), 1)) : r("", !0)]))]), a("div", gi, [M.value ? (v(), n(xn, {
				key: 0,
				files: l.projectFiles,
				selected: l.activeFile || "",
				"mapped-files": N.value,
				onSelect: u[14] ||= (e) => t.$emit("selectFile", e)
			}, null, 8, [
				"files",
				"selected",
				"mapped-files"
			])) : r("", !0), c(Ze, {
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
			])])], 2), l.mode === "editor" ? r("", !0) : (v(), i("div", _i, [a("div", vi, [a("span", null, C(D.value), 1), w.value ? (v(), i("button", {
				key: 0,
				class: "run-code-btn",
				disabled: l.isLoading || l.isReplayMode,
				onClick: T
			}, " Run " + C(_.value), 9, yi)) : r("", !0)]), a("div", { class: p(["pane-body", { "with-file-tree": j.value }]) }, [l.mode === "run" || l.mode === "debug" || l.mode === "replay" ? (v(), n(qt, {
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
			])) : l.mode === "trans" ? (v(), i(e, { key: 1 }, [j.value ? (v(), n(xn, {
				key: 0,
				files: l.transFiles || [],
				selected: l.selectedTransFile || "",
				onSelect: I
			}, null, 8, ["files", "selected"])) : r("", !0), c(yt, {
				code: l.transpiledCode,
				language: k.value,
				"highlight-lines": l.highlightLines,
				onLineClick: P
			}, null, 8, [
				"code",
				"language",
				"highlight-lines"
			])], 64)) : r("", !0)], 2)]))]), L.value ? (v(), i("div", bi, [a("div", xi, [a("span", null, C(R.value), 1)]), a("div", Si, [c(Tt, {
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
			]), (l.isDebugging || l.isReplayMode) && l.debugState ? (v(), n(Zr, {
				key: 0,
				state: l.debugState
			}, null, 8, ["state"])) : r("", !0)])])) : r("", !0)])
		]));
	}
}), [["__scopeId", "data-v-6b1ae17a"]]), wi = "/api", Ti = "auto-playground:state", Ei = "// Welcome to Auto Playground!\nfn add(a int, b int) int {\n    a + b\n}\n\nlet result = add(3, 4)\nprint(result)";
function Di() {
	let e = window.location.hash;
	if (e.startsWith("#share=")) try {
		let t = atob(decodeURIComponent(e.slice(7))), n = JSON.parse(t);
		if (n.source) return n;
	} catch {}
	try {
		let e = localStorage.getItem(Ti);
		if (e) return JSON.parse(e);
	} catch {}
	return {};
}
function Oi(e) {
	try {
		localStorage.setItem(Ti, JSON.stringify(e));
	} catch {}
}
function ki() {
	let e = Di(), n = y(e.source ?? Ei), r = y(""), i = y(""), a = y(""), o = y(0), s = y([]), c = y(null), l = y(!1), u = y(e.activeTab ?? "rust"), d = y(""), f = y(e.projectDir), p = y(e.projectFiles ?? []), m = y(e.activeFile ?? "");
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
			let e = _({ source: n.value }), t = await (await fetch(`${wi}/run`, {
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
				let e = await kt(t);
				r.value = e.stdout, i.value = e.stderr, o.value = 0;
			} else {
				let n = await (await fetch(`${wi}/run_code`, {
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
			}), r = await (await fetch(`${wi}/trans`, {
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
		Oi({
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
function Ai() {
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
function ji() {
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
var Mi = /* @__PURE__ */ l({
	__name: "AutoPlaygroundFull",
	props: { noteMeta: {} },
	setup(n, { expose: r }) {
		let { source: o, stdout: s, stderr: l, resultCode: u, timeMs: d, bytecode: f, bytecodeMeta: m, isLoading: h, activeTab: b, transpiledCode: x, transFiles: S, selectedTransFile: T, projectFiles: E, activeFile: D, highlightedOutputLines: k, highlightedSourceLine: A, mappedSourceFiles: j, run: M, transpile: N, runCode: P, selectTransFile: F, selectFile: I, loadExample: L, highlightOutputLine: R, share: z, shareToast: B } = ki(), V = Ai(), H = ji(), U = y([]), W = y("editor"), G = y("rust"), K = t(() => H.isActive.value ? H.currentState.value : V.state.value), q = t(() => H.isActive.value ? H.bytecode.value : V.bytecode.value), ee = t(() => H.isActive.value ? H.meta.value : V.meta.value), te = t(() => W.value === "run" ? f.value : q.value), ne = t(() => {
			let e = {};
			for (let t of te.value) t.line !== void 0 && (e[t.line] || (e[t.line] = []), e[t.line].push(t.offset));
			return e;
		}), re = t(() => {
			let e = {};
			for (let t of te.value) t.line !== void 0 && (e[t.offset] = t.line);
			return e;
		}), J = y(null), ie = t(() => A.value ? ne.value[A.value] : void 0), Y = t(() => J.value ? ne.value[J.value] : void 0);
		function X(e) {
			A.value = e;
		}
		function Z(e) {
			J.value = e;
		}
		function ae() {
			J.value = null;
		}
		O(() => V.state.value, (e) => {
			e?.status === "finished" && (s.value = e.stdout || "", u.value = e.result || "", l.value = e.stderr || "");
		}), O(() => V.isDebugging.value, (e) => {
			!e && W.value === "debug" && V.state.value?.status !== "finished" && (W.value = "editor");
		}), O(() => H.isActive.value, (e) => {
			!e && W.value === "replay" && (W.value = "editor");
		});
		async function oe() {
			W.value = "run", s.value = "", l.value = "", u.value = "", await M();
		}
		async function se() {
			W.value = "trans", await N(G.value), b.value = G.value;
		}
		async function ce(e) {
			await P(e);
		}
		function Q() {
			V.isDebugging.value || (W.value = "debug", H.stop(), V.connect(o.value, U.value));
		}
		function le() {
			V.isRecording.value ? V.stopRecording() : V.startRecording(o.value, U.value);
		}
		function ue(e) {
			V.sendCommand(e);
		}
		function de(e) {
			let t = re.value[e];
			t && X(t);
		}
		function fe(e) {
			U.value = e, V.setBreakpoints(e);
		}
		async function pe() {
			let e = document.createElement("input");
			e.type = "file", e.accept = ".autoreplay,.json", e.onchange = async () => {
				let t = e.files?.[0];
				if (t) try {
					let e = await t.text(), n = JSON.parse(e);
					V.stop(), H.load(n), W.value = "replay";
				} catch (e) {
					alert("Failed to load replay file: " + e.message);
				}
			}, e.click();
		}
		function me(e) {
			L(e), W.value = "editor";
		}
		function he(e) {
			if (H.isActive.value) {
				switch (e.key) {
					case "ArrowRight":
						e.preventDefault(), H.stepForward();
						break;
					case "ArrowLeft":
						e.preventDefault(), H.stepBackward();
						break;
					case " ":
						e.preventDefault(), H.isPlaying.value ? H.pause() : H.play();
						break;
				}
				return;
			}
			if (V.isDebugging.value) switch (e.key) {
				case "F5":
					e.preventDefault(), ue(e.shiftKey ? "stop" : "continue");
					break;
				case "F10":
					e.preventDefault(), ue("step_over");
					break;
				case "F11":
					e.preventDefault(), ue(e.shiftKey ? "step_out" : "step");
					break;
			}
		}
		return g(() => {
			window.addEventListener("keydown", he), window.__loadReplayForTest__ = (e) => {
				H.load(e), W.value = "replay";
			};
		}), _(() => {
			window.removeEventListener("keydown", he);
		}), r({ loadExample: me }), (t, r) => (v(), i(e, null, [c(Ci, {
			"note-meta": n.noteMeta,
			source: w(o),
			"is-loading": w(h),
			mode: W.value,
			"trans-target": G.value,
			"onUpdate:transTarget": r[0] ||= (e) => G.value = e,
			stdout: w(s),
			stderr: w(l),
			"result-code": w(u),
			"time-ms": w(d),
			"transpiled-code": w(x),
			"trans-files": w(S),
			"selected-trans-file": w(T),
			"project-files": w(E),
			"active-file": w(D),
			"mapped-source-files": w(j),
			"highlight-lines": w(k),
			"on-run": oe,
			"on-trans": se,
			"on-run-code": ce,
			"on-debug": Q,
			"on-select-trans-file": w(F),
			"on-output-line-click": w(R),
			"is-debugging": w(V).isDebugging.value,
			"is-paused": w(V).state.value?.status === "paused",
			"is-recording": w(V).isRecording.value,
			"has-recording": !!w(V).recording.value,
			bytecode: te.value,
			"bytecode-meta": W.value === "run" ? w(m) : ee.value,
			"debug-state": K.value,
			"current-source-line": J.value,
			"highlighted-offsets": Y.value,
			"selected-offsets": ie.value,
			"selected-source-line": w(A),
			breakpoints: U.value,
			"current-debug-line": K.value?.line ?? null,
			"is-replay-mode": w(H).isActive.value,
			"replay-current-index": w(H).currentIndex.value,
			"replay-total-frames": w(H).totalFrames.value,
			"is-replay-playing": w(H).isPlaying.value,
			"onUpdate:source": r[1] ||= (e) => o.value = e,
			onLoadExample: me,
			onSelectFile: w(I),
			onShare: w(z),
			onDebugCommand: ue,
			onToggleRecord: le,
			onExportRecording: w(V).exportRecording,
			onLineClick: X,
			"on-highlight-line": Z,
			"on-clear-highlight": ae,
			onOffsetClick: de,
			onBreakpointsChange: fe,
			onLoadReplay: pe,
			onReplayPlay: w(H).play,
			onReplayPause: w(H).pause,
			onReplayStepForward: w(H).stepForward,
			onReplayStepBackward: w(H).stepBackward,
			onReplaySeek: w(H).seek
		}, null, 8, /* @__PURE__ */ "note-meta.source.is-loading.mode.trans-target.stdout.stderr.result-code.time-ms.transpiled-code.trans-files.selected-trans-file.project-files.active-file.mapped-source-files.highlight-lines.on-select-trans-file.on-output-line-click.is-debugging.is-paused.is-recording.has-recording.bytecode.bytecode-meta.debug-state.current-source-line.highlighted-offsets.selected-offsets.selected-source-line.breakpoints.current-debug-line.is-replay-mode.replay-current-index.replay-total-frames.is-replay-playing.onSelectFile.onShare.onExportRecording.onReplayPlay.onReplayPause.onReplayStepForward.onReplayStepBackward.onReplaySeek".split(".")), a("div", { class: p(["toast", { visible: w(B).visible }]) }, C(w(B).message), 3)], 64));
	}
}), Ni = { class: "nx-sidebar" }, Pi = {
	key: 0,
	class: "nx-brand"
}, Fi = { class: "nx-search" }, Ii = ["value"], Li = ["onClick"], Ri = ["title"], zi = { class: "nx-count" }, Bi = { class: "nx-section-body" }, Vi = {
	key: 0,
	class: "nx-notes"
}, Hi = ["title", "onClick"], Ui = ["onClick", "title"], Wi = { class: "nx-group-title" }, Gi = { class: "nx-count" }, Ki = { class: "nx-notes nx-notes-sub" }, qi = ["title", "onClick"], Ji = {
	key: 0,
	class: "nx-empty"
}, Yi = /* @__PURE__ */ $(/* @__PURE__ */ l({
	__name: "NotesSidebar",
	props: /* @__PURE__ */ d({
		groups: {},
		searching: { type: Boolean },
		activeNoteId: {},
		title: {}
	}, {
		query: { default: "" },
		queryModifiers: {}
	}),
	emits: /* @__PURE__ */ d(["select"], ["update:query"]),
	setup(n, { emit: o }) {
		let s = n, l = T(n, "query"), u = o, d = t(() => {
			let e = s.groups.filter((e) => e.id === "demo"), t = s.groups.filter((e) => e.id.startsWith("book-")), n = s.groups.filter((e) => e.id !== "demo" && !e.id.startsWith("book-")), r = [], i = (e, t, n) => {
				n.length !== 0 && r.push({
					id: e,
					title: t,
					groups: n,
					noteCount: n.reduce((e, t) => e + t.notes.length, 0)
				});
			};
			return i("demo", "Playground Demo", e), i("books", "书籍示例", t), i("tests", "测试用例", n), r;
		}), f = y(/* @__PURE__ */ new Set()), m = y(/* @__PURE__ */ new Set());
		function h(e) {
			return s.searching || f.value.has(e);
		}
		function g(e) {
			return s.searching || m.value.has(e);
		}
		function _(e, t) {
			let n = new Set(e);
			return n.has(t) ? n.delete(t) : n.add(t), n;
		}
		function x(e) {
			f.value = _(f.value, e);
		}
		function S(e) {
			m.value = _(m.value, e);
		}
		function E(e) {
			return e === "demo" ? "demo" : e.startsWith("book-") ? "books" : e ? "tests" : null;
		}
		return O(() => s.activeNoteId, (e) => {
			if (!e) return;
			let t = s.groups.find((t) => t.notes.some((t) => t.id === e));
			if (!t) return;
			let n = E(t.id);
			n && !f.value.has(n) && (f.value = _(f.value, n)), n !== "demo" && !m.value.has(t.id) && (m.value = _(m.value, t.id)), requestAnimationFrame(() => {
				document.querySelector(".nx-note-btn.active")?.scrollIntoView({ block: "nearest" });
			});
		}), (t, o) => (v(), i("aside", Ni, [
			n.title ? (v(), i("div", Pi, C(n.title), 1)) : r("", !0),
			a("div", Fi, [c(w(De), {
				size: 14,
				class: "nx-search-icon"
			}), a("input", {
				class: "nx-search-input",
				type: "text",
				placeholder: "搜索标题或标签…",
				value: l.value,
				onInput: o[0] ||= (e) => l.value = e.target.value
			}, null, 40, Ii)]),
			c(Qe, { class: "nx-tree" }, {
				default: k(() => [(v(!0), i(e, null, b(d.value, (t) => (v(), i("section", {
					key: t.id,
					class: "nx-group"
				}, [a("button", {
					class: "nx-group-head",
					onClick: (e) => x(t.id)
				}, [
					c(w(he), {
						size: 13,
						class: p(["nx-chev", { open: h(t.id) }])
					}, null, 8, ["class"]),
					a("span", {
						class: "nx-group-title",
						title: t.title
					}, C(t.title), 9, Ri),
					a("span", zi, C(t.noteCount), 1)
				], 8, Li), A(a("div", Bi, [t.id === "demo" ? (v(), i("ul", Vi, [(v(!0), i(e, null, b(t.groups[0]?.notes ?? [], (e) => (v(), i("li", { key: e.id }, [a("button", {
					class: p(["nx-note-btn", { active: e.id === n.activeNoteId }]),
					title: e.id,
					onClick: (t) => u("select", e.id)
				}, C(e.title), 11, Hi)]))), 128))])) : (v(!0), i(e, { key: 1 }, b(t.groups, (t) => (v(), i("section", {
					key: t.id,
					class: "nx-group nx-subgroup"
				}, [a("button", {
					class: "nx-group-head",
					onClick: (e) => S(t.id),
					title: `${t.id} · ${t.source}`
				}, [
					c(w(he), {
						size: 13,
						class: p(["nx-chev", { open: g(t.id) }])
					}, null, 8, ["class"]),
					a("span", Wi, C(t.title), 1),
					a("span", Gi, C(t.notes.length), 1)
				], 8, Ui), A(a("ul", Ki, [(v(!0), i(e, null, b(t.notes, (e) => (v(), i("li", { key: e.id }, [a("button", {
					class: p(["nx-note-btn", { active: e.id === n.activeNoteId }]),
					title: e.id,
					onClick: (t) => u("select", e.id)
				}, C(e.title), 11, qi)]))), 128))], 512), [[D, g(t.id)]])]))), 128))], 512), [[D, h(t.id)]])]))), 128)), n.groups.length === 0 ? (v(), i("p", Ji, "无匹配笔记")) : r("", !0)]),
				_: 1
			})
		]));
	}
}), [["__scopeId", "data-v-d959301e"]]);
//#endregion
//#region src/composables/useNotes.ts
function Xi(e = {}) {
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
var Zi = { class: "notes-explorer" }, Qi = { class: "nx-main" }, $i = {
	key: 0,
	class: "nx-state"
}, ea = {
	key: 1,
	class: "nx-state nx-error"
}, ta = {
	key: 2,
	class: "nx-state"
}, na = {
	key: 3,
	class: "nx-state"
}, ra = {
	key: 4,
	class: "nx-note"
}, ia = { class: "nx-note-header" }, aa = { class: "nx-title-row" }, oa = { class: "nx-note-title" }, sa = {
	key: 0,
	class: "nx-kind-chip"
}, ca = { class: "nx-meta-row" }, la = ["href", "title"], ua = { class: "nx-source-path" }, da = {
	key: 0,
	class: "nx-standalone-warn",
	title: "含 import/use 顶层声明，不可独立运行"
}, fa = {
	key: 0,
	class: "nx-desc"
}, pa = "#/notes/", ma = /* @__PURE__ */ $(/* @__PURE__ */ l({
	__name: "NotesExplorer",
	props: {
		base: { default: "/playground-data/notes.json" },
		apiBase: { default: "" },
		repoBase: { default: "https://github.com/auto-stack/auto-lang" },
		ideMode: {
			type: Boolean,
			default: !1
		}
	},
	emits: ["ide-mode"],
	setup(e, { emit: o }) {
		let l = e, u = o, { groups: d, flatNotes: f, byId: m, isLoading: _, error: b, fetchNotes: x, search: S } = Xi({ base: l.base }), T = y(""), E = y(null), D = y(null), k = t(() => E.value ? m.value.get(E.value) ?? null : null), A = t(() => {
			if (!T.value.trim()) return d.value;
			let e = /* @__PURE__ */ new Map();
			for (let { note: t, group: n } of S(T.value)) e.has(n.id) || e.set(n.id, []), e.get(n.id).push(t);
			return d.value.map((t) => ({
				...t,
				notes: e.get(t.id) ?? []
			})).filter((e) => e.notes.length > 0);
		}), j = t(() => {
			let e = k.value?.note;
			return e ? `${l.repoBase.replace(/\/$/, "")}/blob/master/${e.sourcePath}` : "";
		}), M = {
			single: "单文件",
			project: "项目",
			fence: "书页围栏"
		}, N = t(() => M[k.value?.note.kind ?? ""] ?? k.value?.note.kind ?? ""), P = t(() => {
			let e = k.value?.note;
			return e ? e.code ?? e.files?.find((e) => e.path === "main.at")?.content ?? "" : "";
		}), F = t(() => {
			let e = k.value?.note;
			if (!e || e.kind !== "project" || !e.files) return null;
			let t = e.sourcePath.match(/examples\/playground-demo\/(.+)\/main\.at$/);
			return t ? t[1] : null;
		});
		function I(e) {
			E.value = e;
		}
		function L() {
			let e = k.value?.note;
			e && u("ide-mode", {
				noteId: e.id,
				source: e.code ?? e.files?.find((e) => e.path === "main.at")?.content ?? "",
				projectDir: F.value ?? void 0,
				files: e.files?.map((e) => ({
					path: e.path,
					source: e.content
				}))
			});
		}
		function R() {
			if (typeof window > "u") return null;
			let e = window.location.hash;
			if (!e.startsWith(pa)) return null;
			try {
				return decodeURIComponent(e.slice(8));
			} catch {
				return null;
			}
		}
		function z(e) {
			if (typeof window > "u") return;
			let t = pa + encodeURIComponent(e).replace(/%2F/gi, "/");
			window.location.hash !== t && window.history.replaceState(null, "", t);
		}
		O(f, (e) => {
			if (E.value || e.length === 0) return;
			let t = R();
			E.value = t && m.value.has(t) ? t : e[0].note.id;
		}), O(E, (e) => {
			e && z(e);
		});
		function B() {
			let e = R();
			e && m.value.has(e) && e !== E.value && (E.value = e);
		}
		function V(e) {
			let t = e instanceof HTMLElement ? e : null;
			return t ? !!t.closest("input, textarea, select, [contenteditable=\"true\"], .cm-editor") : !1;
		}
		function H() {
			return A.value.flatMap((e) => e.notes.map((e) => e.id));
		}
		function U(e) {
			let t = H();
			if (t.length === 0) return;
			let n = E.value ? t.indexOf(E.value) : -1;
			E.value = t[n === -1 ? 0 : Math.min(t.length - 1, Math.max(0, n + e))];
		}
		function W(e) {
			if ((e.ctrlKey || e.metaKey) && e.key === "Enter") {
				e.preventDefault(), D.value?.run();
				return;
			}
			if (e.key === "ArrowUp" || e.key === "ArrowDown") {
				if (V(e.target)) return;
				e.preventDefault(), U(e.key === "ArrowDown" ? 1 : -1);
			}
		}
		return g(() => {
			window.addEventListener("hashchange", B), window.addEventListener("keydown", W);
		}), h(() => {
			window.removeEventListener("hashchange", B), window.removeEventListener("keydown", W);
		}), (t, o) => (v(), i("div", Zi, [c(Yi, {
			class: "nx-side",
			groups: A.value,
			searching: !!T.value.trim(),
			"active-note-id": E.value,
			query: T.value,
			"onUpdate:query": o[0] ||= (e) => T.value = e,
			onSelect: I
		}, null, 8, [
			"groups",
			"searching",
			"active-note-id",
			"query"
		]), a("main", Qi, [w(_) ? (v(), i("div", $i, "正在加载笔记清单…")) : w(b) ? (v(), i("div", ea, [a("p", null, "笔记清单加载失败：" + C(w(b)), 1), a("button", {
			class: "nx-retry",
			onClick: o[1] ||= (e) => w(x)()
		}, [c(w(Ee), { size: 13 }), o[2] ||= s(" 重试 ", -1)])])) : w(f).length === 0 ? (v(), i("div", ta, "笔记清单为空。")) : k.value ? (v(), i("article", ra, [a("header", ia, [
			a("div", aa, [
				a("h2", oa, C(k.value.note.title), 1),
				a("span", { class: p(["nx-badge", `t-${k.value.note.sourceType}`]) }, C(k.value.note.sourceType), 3),
				k.value.note.kind === "single" ? r("", !0) : (v(), i("span", sa, C(N.value), 1))
			]),
			a("div", ca, [a("a", {
				class: "nx-source-chip",
				href: j.value,
				target: "_blank",
				rel: "noopener",
				title: k.value.note.sourcePath
			}, [
				c(w(Se), { size: 12 }),
				a("span", ua, C(k.value.note.sourcePath), 1),
				c(w(xe), { size: 11 })
			], 8, la), k.value.note.standalone ? r("", !0) : (v(), i("span", da, " 依赖模块 "))]),
			k.value.note.description ? (v(), i("details", fa, [o[3] ||= a("summary", null, "说明", -1), a("p", null, C(k.value.note.description), 1)])) : r("", !0)
		]), (v(), n(cr, {
			ref_key: "card",
			ref: D,
			key: k.value.note.id,
			code: P.value,
			"api-base": e.apiBase,
			"note-id": k.value.note.id,
			"expected-output": k.value.note.expectedOutput,
			"expected-kind": k.value.note.expectedKind,
			files: k.value.note.files,
			"project-dir": F.value,
			"ide-mode": l.ideMode,
			height: "auto",
			onIdeMode: L
		}, null, 8, [
			"code",
			"api-base",
			"note-id",
			"expected-output",
			"expected-kind",
			"files",
			"project-dir",
			"ide-mode"
		]))])) : (v(), i("div", na, "从左侧选择一条笔记开始浏览。"))])]));
	}
}), [["__scopeId", "data-v-73cf3f1f"]]);
//#endregion
export { lr as AutoPlayground, Mi as AutoPlaygroundFull, qt as BytecodePanel, Ze as CodeEditor, yt as CodePreview, Tt as ConsoleOutput, Qt as ExampleSelector, vn as ExpectedOutputPanel, ma as NotesExplorer, Yi as NotesSidebar, cr as PlaygroundCard, Ci as PlaygroundLayout, Qe as ScrollArea, Bt as SnippetRunner, Re as autoLanguage, Ai as useDebugger, Xi as useNotes, Mt as usePlayground, ki as usePlaygroundFull, ji as useReplayPlayer };
