import { Fragment as e, computed as t, createBlock as n, createCommentVNode as r, createElementBlock as i, createElementVNode as a, createStaticVNode as o, createTextVNode as s, createVNode as c, defineComponent as l, h as u, isRef as d, normalizeClass as f, normalizeStyle as p, onMounted as m, onUnmounted as h, openBlock as g, ref as _, renderList as v, toDisplayString as y, unref as b, vModelCheckbox as x, vModelSelect as S, watch as C, withDirectives as w } from "vue";
import { Compartment as T, EditorState as E, RangeSetBuilder as D, StateEffect as O, StateField as k } from "@codemirror/state";
import { Decoration as A, EditorView as j, GutterMarker as M, gutter as N, highlightActiveLine as P, keymap as F, lineNumbers as I } from "@codemirror/view";
import { defaultKeymap as L, history as R, historyKeymap as z, indentWithTab as B } from "@codemirror/commands";
import { oneDark as V } from "@codemirror/theme-one-dark";
import { StreamLanguage as H } from "@codemirror/language";
//#region \0rolldown/runtime.js
var U = Object.create, W = Object.defineProperty, G = Object.getOwnPropertyDescriptor, ee = Object.getOwnPropertyNames, te = Object.getPrototypeOf, K = Object.prototype.hasOwnProperty, ne = (e, t) => () => (t || (e((t = { exports: {} }).exports, t), e = null), t.exports), q = (e, t, n, r) => {
	if (t && typeof t == "object" || typeof t == "function") for (var i = ee(t), a = 0, o = i.length, s; a < o; a++) s = i[a], !K.call(e, s) && s !== n && W(e, s, {
		get: ((e) => t[e]).bind(null, s),
		enumerable: !(r = G(t, s)) || r.enumerable
	});
	return e;
}, J = (e, t, n) => (n = e == null ? {} : U(te(e)), q(t || !e || !e.__esModule ? W(n, "default", {
	value: e,
	enumerable: !0
}) : n, e)), Y = (e) => e.replace(/([a-z0-9])([A-Z])/g, "$1-$2").toLowerCase(), re = {
	xmlns: "http://www.w3.org/2000/svg",
	width: 24,
	height: 24,
	viewBox: "0 0 24 24",
	fill: "none",
	stroke: "currentColor",
	"stroke-width": 2,
	"stroke-linecap": "round",
	"stroke-linejoin": "round"
}, ie = ({ size: e, strokeWidth: t = 2, absoluteStrokeWidth: n, color: r, iconNode: i, name: a, class: o, ...s }, { slots: c }) => u("svg", {
	...re,
	width: e || re.width,
	height: e || re.height,
	stroke: r || re.stroke,
	"stroke-width": n ? Number(t) * 24 / Number(e) : t,
	class: ["lucide", `lucide-${Y(a ?? "icon")}`],
	...s
}, [...i.map((e) => u(...e)), ...c.default ? [c.default()] : []]), X = (e, t) => (n, { slots: r }) => u(ie, {
	...n,
	iconNode: t,
	name: e
}, r), ae = X("ArrowDownIcon", [["path", {
	d: "M12 5v14",
	key: "s699le"
}], ["path", {
	d: "m19 12-7 7-7-7",
	key: "1idqje"
}]]), oe = X("ArrowUpIcon", [["path", {
	d: "m5 12 7-7 7 7",
	key: "hav0vg"
}], ["path", {
	d: "M12 19V5",
	key: "x0mq9r"
}]]), se = X("BugIcon", [
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
]), ce = X("CheckIcon", [["path", {
	d: "M20 6 9 17l-5-5",
	key: "1gmf2c"
}]]), le = X("CodeXmlIcon", [
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
]), ue = X("CopyIcon", [["rect", {
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
}]]), de = X("LoaderCircleIcon", [["path", {
	d: "M21 12a9 9 0 1 1-6.219-8.56",
	key: "13zald"
}]]), fe = X("PlayIcon", [["polygon", {
	points: "6 3 20 12 6 21 6 3",
	key: "1oa8hb"
}]]), pe = X("Share2Icon", [
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
]), me = X("SkipForwardIcon", [["polygon", {
	points: "5 4 15 12 5 20 5 4",
	key: "16p6eg"
}], ["line", {
	x1: "19",
	x2: "19",
	y1: "5",
	y2: "19",
	key: "futhcm"
}]]), he = X("SquareIcon", [["rect", {
	width: "18",
	height: "18",
	x: "3",
	y: "3",
	rx: "2",
	key: "afitv7"
}]]), ge = new Set(/* @__PURE__ */ "fn.let.mut.const.var.type.union.enum.tag.alias.spec.ext.static.shared.impl.node.if.else.for.break.continue.loop.is.in.on.as.to.return.next.view.move.copy.take.hold.true.false.nil.null.None.Some.Ok.Err.task.spawn.await.reply.go.use.pac.super.dep.has.and.or.routes.outlet.link.route.nav.grid".split(".")), _e = new Set(/* @__PURE__ */ "int.uint.byte.i8.i16.i64.u8.u16.u64.usize.float.double.bool.char.void.str.String.cstr.Handle.linear.List.Map.Set.Option.Result.Link".split("."));
function Z(e) {
	return e >= "0" && e <= "9";
}
function ve(e) {
	return Z(e) || e >= "a" && e <= "f" || e >= "A" && e <= "F";
}
function ye(e) {
	return /[\p{L}_]/u.test(e);
}
function be(e) {
	return /[\p{L}\p{N}_-]/u.test(e);
}
var xe = H.define({
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
		return Z(n) || n === "." && Z(e.string[e.pos + 1] || "") || n === "0" && (e.string[e.pos + 1] === "x" || e.string[e.pos + 1] === "b") ? Se(e) : ye(n) ? Ce(e) : n === "#" ? (e.next(), e.match("if") || e.match("for") || e.match("is") ? "keyword" : e.match("[") ? "meta" : e.match("{") ? "macroName" : "operator") : n === "@" ? (e.next(), ye(e.peek() || "") && e.eatWhile(be), "attributeName") : e.match("==") || e.match("!=") || e.match("<=") || e.match(">=") || e.match("->") || e.match("=>") || e.match("..=") || e.match("??") || e.match("?.") || e.match(".?") || e.match("&&") || e.match("||") || e.match("+=") || e.match("-=") || e.match("*=") || e.match("/=") || e.match("%=") || n === "." && e.match("..") ? "operator" : n === "." && /[a-zA-Z]/.test(e.string[e.pos + 1] || "") ? (e.next(), e.eatWhile(/[a-zA-Z]/), "propertyName") : "+-*/%=<>!&|~:;,.[](){}".indexOf(n) >= 0 ? (e.next(), "operator") : (e.next(), null);
	},
	languageData: { commentTokens: {
		line: "//",
		block: {
			open: "/*",
			close: "*/"
		}
	} }
});
function Se(e) {
	let t = e.pos, n = e.peek();
	return n === "0" && (e.string[t + 1] === "x" || e.string[t + 1] === "X") ? (e.next(), e.next(), e.eatWhile(ve), e.eatWhile(/[uUiIfFdD]/), "number") : n === "0" && (e.string[t + 1] === "b" || e.string[t + 1] === "B") ? (e.next(), e.next(), e.eatWhile(/[01]/), "number") : (e.eatWhile(Z), e.eatWhile(/[_]/), e.eatWhile(Z), e.peek() === "." && Z(e.string[e.pos + 1] || "") && (e.next(), e.eatWhile(Z), e.eatWhile(/[_]/), e.eatWhile(Z)), (e.peek() === "e" || e.peek() === "E") && (e.next(), (e.peek() === "-" || e.peek() === "+") && e.next(), e.eatWhile(Z)), e.match("usize") || e.match("i64") || e.match("i16") || e.match("i8") || e.match("u64") || e.match("u16") || e.match("u8") || e.match("u") || e.match("f") || e.match("d"), "number");
}
function Ce(e) {
	e.eatWhile(be);
	let t = e.current();
	return ge.has(t) ? "keyword" : _e.has(t) ? "typeName" : e.string.slice(e.pos).trimStart()[0] === "(" ? "function" : "variableName";
}
//#endregion
//#region src/components/CodeEditor.vue?vue&type=script&setup=true&lang.ts
var we = /* @__PURE__ */ l({
	__name: "CodeEditor",
	props: {
		modelValue: {},
		onRun: { type: Function },
		isDebugging: { type: Boolean },
		breakpoints: {},
		currentDebugLine: {},
		highlightedSourceLine: {},
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
		let n = e, r = t, a = _(), o = null, s = new T(), c = O.define(), l = k.define({
			create() {
				return /* @__PURE__ */ new Set();
			},
			update(e, t) {
				for (let n of t.effects) if (n.is(c)) {
					let t = n.value, r = new Set(e);
					return r.has(t) ? r.delete(t) : r.add(t), r;
				}
				return e;
			}
		});
		class u extends M {
			eq(e) {
				return e instanceof u;
			}
			toDOM() {
				let e = document.createElement("div");
				return e.style.width = "10px", e.style.height = "10px", e.style.borderRadius = "50%", e.style.background = "#e51400", e.className = "cm-breakpoint-marker", e;
			}
		}
		class d extends M {
			eq(e) {
				return e instanceof d;
			}
			toDOM() {
				let e = document.createElement("div");
				return e.style.width = "10px", e.style.height = "10px", e.style.borderRadius = "50%", e.style.border = "1.5px solid #e51400", e.style.background = "transparent", e.style.opacity = "0", e.style.transition = "opacity 0.15s ease", e.className = "cm-empty-circle-marker", e;
			}
		}
		class f extends M {
			eq(e) {
				return e instanceof f;
			}
			toDOM() {
				let e = document.createElement("div");
				return e.style.width = "22px", e.className = "cm-breakpoint-spacer", e;
			}
		}
		let p = [l, N({
			class: "cm-breakpoint-gutter",
			markers(e) {
				let t = new D(), n = e.state.field(l);
				for (let r = 1; r <= e.state.doc.lines; r++) {
					let i = e.state.doc.line(r), a = n.has(r) ? new u() : new d();
					t.add(i.from, i.from, a);
				}
				return t.finish();
			},
			initialSpacer() {
				return new f();
			},
			domEventHandlers: { mousedown(e, t) {
				let n = e.state.doc.lineAt(t.from).number;
				e.dispatch({ effects: c.of(n) });
				let i = e.state.field(l);
				return r("breakpointsChange", Array.from(i)), r("line-click", n), !0;
			} }
		})], v = O.define(), y = O.define(), b = k.define({
			create() {
				return A.none;
			},
			update(e, t) {
				for (let e of t.effects) if (e.is(v)) {
					if (e.value === null || e.value <= 0) return A.none;
					let n = t.state.doc.line(e.value);
					return A.set([A.line({ class: "cm-debug-current-line" }).range(n.from)]);
				}
				return e.map(t.changes);
			},
			provide: (e) => j.decorations.from(e)
		}), x = k.define({
			create() {
				return A.none;
			},
			update(e, t) {
				for (let e of t.effects) if (e.is(y)) {
					if (e.value === null || e.value <= 0) return A.none;
					let n = t.state.doc.line(e.value);
					return A.set([A.line({ class: "cm-cross-highlight-line" }).range(n.from)]);
				}
				return e.map(t.changes);
			},
			provide: (e) => j.decorations.from(e)
		}), S = O.define(), w = [
			b,
			x,
			k.define({
				create() {
					return A.none;
				},
				update(e, t) {
					for (let e of t.effects) if (e.is(S)) {
						if (!e.value || e.value.length === 0) return A.none;
						let n = e.value.filter((e) => e > 0 && e <= t.state.doc.lines).map((e) => {
							let n = t.state.doc.line(e);
							return A.line({ class: "cm-error-line" }).range(n.from);
						});
						return A.set(n);
					}
					return e.map(t.changes);
				},
				provide: (e) => j.decorations.from(e)
			}),
			j.baseTheme({
				".cm-debug-current-line": {
					backgroundColor: "#0e639c40",
					borderLeft: "3px solid #0e639c"
				},
				".cm-cross-highlight-line": {
					backgroundColor: "#7b4a0e40",
					borderLeft: "3px solid #ff9d00"
				},
				".cm-error-line": {
					backgroundColor: "#f38ba822",
					borderLeft: "3px solid #f38ba8"
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
		function H() {
			return [...p, ...w];
		}
		return m(() => {
			if (!a.value) return;
			let e = [
				I(),
				P(),
				R(),
				F.of([
					...L,
					...z,
					B
				]),
				xe,
				V,
				j.updateListener.of((e) => {
					e.docChanged && !n.readOnly && r("update:modelValue", e.state.doc.toString());
				}),
				s.of(n.isDebugging ? H() : [])
			];
			n.readOnly && e.push(j.editable.of(!1)), n.onRun && e.push(F.of([{
				key: "Ctrl-Enter",
				run: () => (n.onRun?.(), !0)
			}])), o = new j({
				state: E.create({
					doc: n.modelValue,
					extensions: e
				}),
				parent: a.value
			});
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
		}), C(() => n.modelValue, (e) => {
			o && o.state.doc.toString() !== e && o.dispatch({ changes: {
				from: 0,
				to: o.state.doc.length,
				insert: e
			} });
		}), C(() => n.isDebugging, (e) => {
			o && o.dispatch({ effects: s.reconfigure(e ? H() : []) });
		}), C(() => n.currentDebugLine, (e) => {
			o && o.dispatch({ effects: v.of(e ?? null) });
		}), C(() => n.highlightedSourceLine, (e) => {
			o && o.dispatch({ effects: y.of(e ?? null) });
		}), C(() => n.errorLines, (e) => {
			o && o.dispatch({ effects: S.of(e ?? []) });
		}), C(() => n.breakpoints, (e) => {
			if (!o) return;
			let t = o.state.field(l, !1);
			if (!t) return;
			let n = Array.from(t), r = e || [], i = r.filter((e) => !n.includes(e)), a = n.filter((e) => !r.includes(e));
			if (i.length === 0 && a.length === 0) return;
			let s = [...i.map((e) => c.of(e)), ...a.map((e) => c.of(e))];
			o.dispatch({ effects: s });
		}, { deep: !0 }), h(() => {
			o?.destroy();
		}), (e, t) => (g(), i("div", {
			ref_key: "editorContainer",
			ref: a,
			class: "editor-container"
		}, null, 512));
	}
}), Q = (e, t) => {
	let n = e.__vccOpts || e;
	for (let [e, r] of t) n[e] = r;
	return n;
}, Te = /* @__PURE__ */ Q(we, [["__scopeId", "data-v-be13e4de"]]), $ = (/* @__PURE__ */ J((/* @__PURE__ */ ne(((e, t) => {
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
	}, ee = {
		begin: "\\.\\s*" + E,
		relevance: 0
	}, te = /* @__PURE__ */ Object.freeze({
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
		METHOD_GUARD: ee,
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
	function K(e, t) {
		e.input[e.index - 1] === "." && t.ignoreMatch();
	}
	function ne(e, t) {
		e.className !== void 0 && (e.scope = e.className, delete e.className);
	}
	function q(e, t) {
		t && e.beginKeywords && (e.begin = "\\b(" + e.beginKeywords.split(" ").join("|") + ")(?!\\.)(?=\\b|\\s)", e.__beforeBegin = K, e.keywords = e.keywords || e.beginKeywords, delete e.beginKeywords, e.relevance === void 0 && (e.relevance = 0));
	}
	function J(e, t) {
		Array.isArray(e.illegal) && (e.illegal = y(...e.illegal));
	}
	function Y(e, t) {
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
		return t ? Number(t) : +!ce(e);
	}
	function ce(e) {
		return X.includes(e.toLowerCase());
	}
	var le = {}, ue = (e) => {
		console.error(e);
	}, de = (e, ...t) => {
		console.log(`WARN: ${e}`, ...t);
	}, fe = (e, t) => {
		le[`${e}/${t}`] || (console.log(`Deprecated as of ${e}. ${t}`), le[`${e}/${t}`] = !0);
	}, pe = /* @__PURE__ */ Error();
	function me(e, t, { key: n }) {
		let r = 0, i = e[n], a = {}, o = {};
		for (let e = 1; e <= t.length; e++) o[e + r] = i[e], a[e + r] = !0, r += b(t[e - 1]);
		e[n] = o, e[n]._emit = a, e[n]._multi = !0;
	}
	function he(e) {
		if (Array.isArray(e.begin)) {
			if (e.skip || e.excludeBegin || e.returnBegin) throw ue("skip, excludeBegin, returnBegin not compatible with beginScope: {}"), pe;
			if (typeof e.beginScope != "object" || e.beginScope === null) throw ue("beginScope must be object"), pe;
			me(e, e.begin, { key: "beginScope" }), e.begin = C(e.begin, { joinWith: "" });
		}
	}
	function ge(e) {
		if (Array.isArray(e.end)) {
			if (e.skip || e.excludeEnd || e.returnEnd) throw ue("skip, excludeEnd, returnEnd not compatible with endScope: {}"), pe;
			if (typeof e.endScope != "object" || e.endScope === null) throw ue("endScope must be object"), pe;
			me(e, e.end, { key: "endScope" }), e.end = C(e.end, { joinWith: "" });
		}
	}
	function _e(e) {
		e.scope && typeof e.scope == "object" && e.scope !== null && (e.beginScope = e.scope, delete e.scope);
	}
	function Z(e) {
		_e(e), typeof e.beginScope == "string" && (e.beginScope = { _wrap: e.beginScope }), typeof e.endScope == "string" && (e.endScope = { _wrap: e.endScope }), he(e), ge(e);
	}
	function ve(e) {
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
				ne,
				Y,
				Z,
				ie
			].forEach((e) => e(n, r)), e.compilerExtensions.forEach((e) => e(n, r)), n.__beforeBegin = null, [
				q,
				J,
				re
			].forEach((e) => e(n, r)), n.isCompiled = !0;
			let s = null;
			return typeof n.keywords == "object" && n.keywords.$pattern && (n.keywords = Object.assign({}, n.keywords), s = n.keywords.$pattern, delete n.keywords.$pattern), s ||= /\w+/, n.keywords &&= oe(n.keywords, e.case_insensitive), a.keywordPatternRe = t(s, !0), r && (n.begin ||= /\B|\b/, a.beginRe = t(a.begin), !n.end && !n.endsWithParent && (n.end = /\B|\b/), n.end && (a.endRe = t(a.end)), a.terminatorEnd = p(a.end) || "", n.endsWithParent && r.terminatorEnd && (a.terminatorEnd += (n.end ? "|" : "") + r.terminatorEnd)), n.illegal && (a.illegalRe = t(n.illegal)), n.contains ||= [], n.contains = [].concat(...n.contains.map(function(e) {
				return be(e === "self" ? n : e);
			})), n.contains.forEach(function(e) {
				o(e, a);
			}), n.starts && o(n.starts, r), a.matcher = i(a), a;
		}
		if (e.compilerExtensions ||= [], e.contains && e.contains.includes("self")) throw Error("ERR: contains `self` is not supported at the top-level of a language.  See documentation.");
		return e.classNameAliases = a(e.classNameAliases || {}), o(e);
	}
	function ye(e) {
		return e ? e.endsWithParent || ye(e.starts) : !1;
	}
	function be(e) {
		return e.variants && !e.cachedVariants && (e.cachedVariants = e.variants.map(function(t) {
			return a(e, { variants: null }, t);
		})), e.cachedVariants ? e.cachedVariants : ye(e) ? a(e, { starts: e.starts ? a(e.starts) : null }) : Object.isFrozen(e) ? a(e) : e;
	}
	var xe = "11.11.1", Se = class extends Error {
		constructor(e, t) {
			super(e), this.name = "HTMLInjectionError", this.html = t;
		}
	}, Ce = i, we = a, Q = Symbol("nomatch"), Te = 7, $ = function(e) {
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
				return t || (de(s.replace("{}", n[1])), de("Falling back to no-highlight mode for this block.", e)), t ? n[1] : "no-highlight";
			}
			return t.split(/\s+/).find((e) => u(e) || N(e));
		}
		function p(e, t, n) {
			let r = "", i = "";
			typeof t == "object" ? (r = e, n = t.ignoreIllegals, i = t.language) : (fe("10.7.0", "highlight(lang, code, ...args) has been deprecated."), fe("10.7.0", "Please use highlight(code, options) instead.\nhttps://github.com/highlightjs/highlight.js/issues/2277"), i = e, r = t), n === void 0 && (n = !0);
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
				if (!i) return Q;
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
					if (e !== Q) return e;
				}
				if (r.type === "illegal" && a === "") return P += "\n", 1;
				if (L > 1e5 && L > r.index * 3) throw /* @__PURE__ */ Error("potential infinite loop, way more iterations than matches");
				return P += a, a.length;
			}
			let D = N(e);
			if (!D) throw ue(s.replace("{}", e)), Error("Unknown language: \"" + e + "\"");
			let O = ve(D), k = "", A = a || O, j = {}, M = new l.__emitter(l);
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
					value: Ce(n),
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
					value: Ce(n),
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
				value: Ce(e),
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
			if (e.children.length > 0 && (l.ignoreUnescapedHTML || (console.warn("One of your code blocks includes unescaped HTML. This is a potentially serious security risk."), console.warn("https://github.com/highlightjs/highlight.js/wiki/security"), console.warn("The element with unescaped HTML:"), console.warn(e)), l.throwUnescapedHTML)) throw new Se("One of your code blocks includes unescaped HTML.", e.innerHTML);
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
			l = we(l, e);
		}
		let E = () => {
			k(), fe("10.6.0", "initHighlighting() deprecated.  Use highlightAll() now.");
		};
		function D() {
			k(), fe("10.6.0", "initHighlightingOnLoad() deprecated.  Use highlightAll() now.");
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
				if (ue("Language definition for '{}' could not be registered.".replace("{}", n)), o) ue(e);
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
			return fe("10.7.0", "highlightBlock will be removed entirely in v12.0"), fe("10.7.0", "Please use highlightElement now."), w(e);
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
			inherit: we,
			addPlugin: L,
			removePlugin: R
		}), e.debugMode = function() {
			o = !1;
		}, e.safeMode = function() {
			o = !0;
		}, e.versionString = xe, e.regex = {
			concat: _,
			lookahead: m,
			either: y,
			optional: g,
			anyNumberOfTimes: h
		};
		for (let e in te) typeof te[e] == "object" && n(te[e]);
		return Object.assign(e, te), e;
	}, Ee = $({});
	Ee.newInstance = () => $({}), t.exports = Ee, Ee.HighlightJS = Ee, Ee.default = Ee;
})))())).default;
//#endregion
//#region node_modules/highlight.js/es/languages/rust.js
function Ee(e) {
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
function De(e) {
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
var Oe = "[A-Za-z$_][0-9A-Za-z$_]*", ke = /* @__PURE__ */ "as.in.of.if.for.while.finally.var.new.function.do.return.void.else.break.catch.instanceof.with.throw.case.default.try.switch.continue.typeof.delete.let.yield.const.class.debugger.async.await.static.import.from.export.extends.using".split("."), Ae = [
	"true",
	"false",
	"null",
	"undefined",
	"NaN",
	"Infinity"
], je = /* @__PURE__ */ "Object.Function.Boolean.Symbol.Math.Date.Number.BigInt.String.RegExp.Array.Float32Array.Float64Array.Int8Array.Uint8Array.Uint8ClampedArray.Int16Array.Int32Array.Uint16Array.Uint32Array.BigInt64Array.BigUint64Array.Set.Map.WeakSet.WeakMap.ArrayBuffer.SharedArrayBuffer.Atomics.DataView.JSON.Promise.Generator.GeneratorFunction.AsyncFunction.Reflect.Proxy.Intl.WebAssembly".split("."), Me = [
	"Error",
	"EvalError",
	"InternalError",
	"RangeError",
	"ReferenceError",
	"SyntaxError",
	"TypeError",
	"URIError"
], Ne = [
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
], Pe = [
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
], Fe = [].concat(Ne, je, Me);
function Ie(e) {
	let t = e.regex, n = (e, { after: t }) => {
		let n = "</" + e[0].slice(1);
		return e.input.indexOf(n, t) !== -1;
	}, r = Oe, i = {
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
		$pattern: Oe,
		keyword: ke,
		literal: Ae,
		built_in: Fe,
		"variable.language": Pe
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
		keywords: { _: [...je, ...Me] }
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
			...Ne,
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
function Le(e) {
	let t = e.regex, n = Ie(e), r = Oe, i = [
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
		$pattern: Oe,
		keyword: ke.concat([
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
		literal: Ae,
		built_in: Fe.concat(i),
		"variable.language": Pe
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
function Re(e) {
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
var ze = () => ({
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
}), Be = { class: "code-preview" }, Ve = { class: "lines-container" }, He = ["onClick"], Ue = { class: "line-number" }, We = ["innerHTML"], Ge = {
	key: 0,
	class: "code-line"
}, Ke = /* @__PURE__ */ Q(/* @__PURE__ */ l({
	__name: "CodePreview",
	props: {
		code: {},
		language: {},
		highlightLines: {}
	},
	emits: ["line-click"],
	setup(n, { emit: o }) {
		$.registerLanguage("rust", Ee), $.registerLanguage("python", De), $.registerLanguage("typescript", Le), $.registerLanguage("c", Re), $.registerLanguage("abt", ze);
		let s = n, c = o, l = {
			rust: "rust",
			python: "python",
			typescript: "typescript",
			c: "c",
			abt: "abt"
		}, u = t(() => {
			if (!s.code) return [""];
			let e = s.language ? l[s.language] : void 0;
			if (!e) return s.code.split("\n");
			try {
				return $.highlight(s.code, { language: e }).value.split("\n");
			} catch {
				return s.code.split("\n");
			}
		});
		function d(e) {
			return s.highlightLines?.includes(e) ?? !1;
		}
		function p(e) {
			c("line-click", e);
		}
		return (t, n) => (g(), i("div", Be, [a("div", Ve, [(g(!0), i(e, null, v(u.value, (e, t) => (g(), i("div", {
			key: t,
			class: f(["code-line", { highlighted: d(t + 1) }]),
			onClick: (e) => p(t + 1)
		}, [a("span", Ue, y(t + 1), 1), a("span", {
			class: "line-content",
			innerHTML: e || " "
		}, null, 8, We)], 10, He))), 128)), u.value.length === 0 ? (g(), i("div", Ge, [...n[0] ||= [a("span", { class: "line-number" }, "1", -1), a("span", { class: "line-content" }, null, -1)]])) : r("", !0)])]));
	}
}), [["__scopeId", "data-v-13928974"]]), qe = { class: "console-output" }, Je = {
	key: 0,
	class: "time-info"
}, Ye = {
	key: 1,
	class: "stdout"
}, Xe = {
	key: 2,
	class: "stderr"
}, Ze = {
	key: 3,
	class: "result"
}, Qe = {
	key: 4,
	class: "empty"
}, $e = /* @__PURE__ */ Q(/* @__PURE__ */ l({
	__name: "ConsoleOutput",
	props: {
		stdout: {},
		stderr: {},
		result: {},
		timeMs: {}
	},
	setup(e) {
		return (t, n) => (g(), i("div", qe, [
			e.timeMs > 0 ? (g(), i("div", Je, "Completed in " + y(e.timeMs) + "ms", 1)) : r("", !0),
			e.stdout ? (g(), i("pre", Ye, y(e.stdout), 1)) : r("", !0),
			e.stderr ? (g(), i("pre", Xe, y(e.stderr), 1)) : r("", !0),
			e.result ? (g(), i("pre", Ze, "Result: " + y(e.result), 1)) : r("", !0),
			!e.stdout && !e.stderr && !e.result ? (g(), i("div", Qe, "Click Run or press Ctrl+Enter to execute")) : r("", !0)
		]));
	}
}), [["__scopeId", "data-v-e77ae24c"]]), et = { class: "bytecode-panel" }, tt = ["onClick"], nt = { class: "offset" }, rt = { class: "mnemonic" }, it = { class: "operands" }, at = /* @__PURE__ */ Q(/* @__PURE__ */ l({
	__name: "BytecodePanel",
	props: {
		bytecode: {},
		currentIp: {},
		highlightedOffsets: {}
	},
	emits: ["offsetClick"],
	setup(t) {
		function n(e) {
			return e.toString(16).padStart(4, "0");
		}
		return (r, o) => (g(), i("div", et, [(g(!0), i(e, null, v(t.bytecode, (e) => (g(), i("div", {
			key: e.offset,
			class: f(["bytecode-line", {
				"is-current": e.offset === t.currentIp,
				"is-highlighted": t.highlightedOffsets?.includes(e.offset),
				"has-source": e.line !== void 0
			}]),
			onClick: (t) => r.$emit("offsetClick", e.offset)
		}, [
			a("span", nt, y(n(e.offset)), 1),
			a("span", rt, y(e.mnemonic), 1),
			a("span", it, y(e.operands), 1)
		], 10, tt))), 128))]));
	}
}), [["__scopeId", "data-v-1232241a"]]), ot = { label: "Single-file" }, st = ["value"], ct = { label: "Projects" }, lt = ["value"], ut = /* @__PURE__ */ Q(/* @__PURE__ */ l({
	__name: "ExampleSelector",
	props: { apiBase: { default: "/api" } },
	emits: ["select"],
	setup(n, { emit: r }) {
		let o = n, s = r, c = _([]), l = _(""), u = t(() => c.value.filter((e) => e.example_type === "single")), d = t(() => c.value.filter((e) => e.example_type === "project"));
		m(async () => {
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
		return (t, n) => w((g(), i("select", {
			class: "example-selector",
			onChange: f,
			"onUpdate:modelValue": n[0] ||= (e) => l.value = e
		}, [
			n[1] ||= a("option", { value: "" }, "Load Example...", -1),
			a("optgroup", ot, [(g(!0), i(e, null, v(u.value, (e) => (g(), i("option", {
				key: e.name,
				value: JSON.stringify(e)
			}, y(e.name), 9, st))), 128))]),
			a("optgroup", ct, [(g(!0), i(e, null, v(d.value, (e) => (g(), i("option", {
				key: e.name,
				value: JSON.stringify(e)
			}, y(e.name), 9, lt))), 128))])
		], 544)), [[S, l.value]]);
	}
}), [["__scopeId", "data-v-578bba03"]]), dt = { class: "file-tree" }, ft = ["onClick"], pt = { class: "file-name" }, mt = /* @__PURE__ */ Q(/* @__PURE__ */ l({
	__name: "FileTree",
	props: {
		files: {},
		selected: {},
		mappedFiles: {}
	},
	emits: ["select"],
	setup(t) {
		return (n, r) => (g(), i("div", dt, [(g(!0), i(e, null, v(t.files, (e) => (g(), i("div", {
			key: e.path,
			class: f(["file-item", {
				active: e.path === t.selected,
				mapped: t.mappedFiles?.includes(e.path)
			}]),
			onClick: (t) => n.$emit("select", e.path)
		}, [a("span", pt, y(e.path), 1)], 10, ft))), 128))]));
	}
}), [["__scopeId", "data-v-cf362a80"]]), ht = !1, gt = null;
function _t() {
	return ht ? Promise.resolve() : gt || (gt = new Promise((e, t) => {
		if (window.ts) {
			ht = !0, e();
			return;
		}
		let n = document.createElement("script");
		n.src = "https://cdn.jsdelivr.net/npm/typescript@5.7.3/lib/typescript.js", n.onload = () => {
			ht = !0, e();
		}, n.onerror = () => t(/* @__PURE__ */ Error("Failed to load TypeScript compiler")), document.head.appendChild(n);
	}), gt);
}
async function vt(e) {
	try {
		await _t();
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
var yt = 500, bt = "// Welcome to Auto Playground!\nfn add(a int, b int) int {\n    a + b\n}\n\nlet result = add(3, 4)\nprint(result)";
function xt(e = {}) {
	let n = e.apiBase ?? "/api", r = e.persistKey ?? "auto-playground:state", i = e.defaultSource ?? bt, a = e.preloadTargets ?? !0;
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
	let c = o(), l = _(c.source ?? i), u = _(""), d = _(""), f = _(""), p = _(0), m = _([]), h = _(!1), g = _(c.activeTab ?? "rust"), v = _(""), y = _(c.liveCompile ?? !0), b = _(c.projectDir), x = _({}), S = _(null), w = _([]), T = _([]), E = _({
		message: "",
		visible: !1
	}), D = null, O = t(() => {
		let e = v.value;
		if (!e) return "";
		let t = x.value[e];
		return t ? t.files.find((e) => e.path === t.selectedFile)?.code ?? t.files[0]?.code ?? "" : "";
	}), k = t(() => {
		let e = v.value;
		return e ? x.value[e]?.files ?? [] : [];
	}), A = t(() => {
		let e = v.value;
		return e ? x.value[e]?.selectedFile ?? "" : "";
	}), j = t(() => ""), M = t(() => {
		let e = v.value, t = /* @__PURE__ */ new Map();
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
		let e = v.value, t = /* @__PURE__ */ new Map();
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
		S.value ? F(S.value) : (w.value = [], T.value = []);
	}
	function F(e) {
		S.value = e;
		let t = j.value, n = M.value.get(t)?.get(e) ?? [];
		T.value = n.map((e) => e.outputFile);
		let r = A.value;
		w.value = n.find((e) => e.outputFile === r)?.outputLines ?? [];
	}
	function I(e, t) {
		let n = N.value.get(e)?.get(t);
		if (!n) {
			R();
			return;
		}
		S.value = n.sourceLine;
		let r = M.value.get(n.sourceFile)?.get(n.sourceLine) ?? [];
		T.value = r.map((e) => e.outputFile), w.value = r.find((t) => t.outputFile === e)?.outputLines ?? [];
	}
	function L(e, t) {
		return N.value.get(e)?.get(t)?.sourceFile;
	}
	function R() {
		S.value = null, w.value = [], T.value = [];
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
				let e = await vt(t);
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
			v.value = e, x.value[e] = {
				files: i,
				fileSourceMaps: a,
				selectedFile: o
			}, P();
		} catch (t) {
			v.value = e, x.value[e] = {
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
		if (g.value = e, v.value = e, !x.value[e] && y.value) {
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
		l.value = e.source, b.value = e.project_dir, u.value = "", d.value = "", f.value = "", m.value = [], S.value = null, w.value = [], T.value = [];
	}
	function G() {
		if (typeof window > "u") return "";
		let e = JSON.stringify({
			source: l.value,
			activeTab: g.value,
			liveCompile: y.value,
			projectDir: b.value
		}), t = "#share=" + encodeURIComponent(btoa(e));
		return window.location.origin + window.location.pathname + t;
	}
	async function ee() {
		let e = G(), t = !1;
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
		E.value = {
			message: t ? "Share link copied to clipboard!" : "Failed to copy link",
			visible: !0
		}, setTimeout(() => {
			E.value.visible = !1;
		}, 2500);
	}
	C(l, () => {
		x.value = {}, y.value && (D && clearTimeout(D), D = setTimeout(() => {
			V(g.value);
		}, yt));
	}), C([
		l,
		g,
		y,
		b
	], ([e, t, n, r]) => {
		s({
			source: e,
			activeTab: t,
			liveCompile: n,
			projectDir: r
		});
	}, { deep: !0 }), a && typeof window < "u" && setTimeout(() => {
		te();
	}, 100);
	async function te() {
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
			x.value[r] && (v.value = r, P());
		} finally {
			h.value = !1;
		}
	}
	let K = _(null), ne = _(null), q = _([]), J = _([]), Y = _(!1);
	async function re() {
		h.value = !0, d.value = "";
		try {
			let e = await (await fetch(`${n}/agent-debug/start`, {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify({ source: l.value })
			})).json();
			K.value = e.session_id, q.value = (e.bytecode || []).map((e) => ({
				offset: e.offset ?? e.idx ?? 0,
				mnemonic: e.mnemonic ?? e.op ?? "",
				operands: e.operands ?? e.args ?? "",
				line: e.line
			})), Y.value = !0, ne.value = null, J.value.length > 0 && await ie(J.value);
		} catch (e) {
			d.value = `Debug start error: ${e.message}`;
		} finally {
			h.value = !1;
		}
	}
	async function ie(e) {
		if (J.value = e, K.value) try {
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
				ne.value = t, t.stdout && (u.value = t.stdout), t.stderr && (d.value = t.stderr), t.result && (f.value = t.result), (t.status === "finished" || t.status === "error") && (Y.value = !1);
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
			K.value = null, ne.value = null, q.value = [], Y.value = !1;
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
		transpiledCode: O,
		transpileTarget: v,
		liveCompile: y,
		projectDir: b,
		transFiles: k,
		selectedTransFile: A,
		highlightedSourceLine: S,
		highlightedOutputLines: w,
		highlightedOutputFiles: T,
		shareToast: E,
		debugSessionId: K,
		debugState: ne,
		bytecode: q,
		breakpoints: J,
		isDebugging: Y,
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
		share: ee,
		debugStart: re,
		debugSetBreakpoints: ie,
		debugCommand: X,
		debugStop: ae
	};
}
//#endregion
//#region src/AutoPlayground.vue?vue&type=script&setup=true&lang.ts
var St = { class: "playground-toolbar" }, Ct = { class: "toolbar-left" }, wt = { class: "toolbar-right" }, Tt = ["disabled"], Et = ["disabled"], Dt = {
	key: 1,
	class: "debug-controls"
}, Ot = ["disabled"], kt = ["disabled"], At = ["disabled"], jt = ["disabled"], Mt = ["disabled"], Nt = {
	key: 4,
	class: "switch-widget",
	title: "Toggle live transpile on edit"
}, Pt = { class: "switch" }, Ft = { class: "playground-body" }, It = { class: "editor-pane" }, Lt = { class: "output-pane" }, Rt = { class: "output-tabs" }, zt = ["onClick"], Bt = ["title"], Vt = { class: "output-content" }, Ht = {
	key: 2,
	class: "output-code-split"
}, Ut = {
	key: 0,
	class: "debug-panel"
}, Wt = {
	key: 0,
	class: "debug-section"
}, Gt = { class: "debug-section-title" }, Kt = { class: "debug-stack" }, qt = {
	key: 1,
	class: "debug-section"
}, Jt = { class: "frame-name" }, Yt = { class: "frame-info" }, Xt = {
	key: 2,
	class: "debug-section"
}, Zt = { class: "debug-locals" }, Qt = { class: "local-idx" }, $t = {
	key: 3,
	class: "debug-registers"
}, en = /* @__PURE__ */ Q(/* @__PURE__ */ l({
	__name: "AutoPlayground",
	props: {
		code: { default: "fn main() {\n    let message = \"Hello from Auto!\"\n    print(message)\n}" },
		apiUrl: { default: "" },
		height: { default: "500px" }
	},
	setup(l) {
		let u = l, m = u.apiUrl ? `${u.apiUrl}/api` : "/api", { source: h, stdout: T, stderr: E, resultCode: D, timeMs: O, isLoading: k, transpiledCode: A, liveCompile: j, transFiles: M, selectedTransFile: N, highlightedOutputLines: P, shareToast: F, debugState: I, bytecode: L, breakpoints: R, isDebugging: z, run: B, switchTab: V, selectTransFile: H, loadExample: U, share: W, highlightOutputLine: G, debugStart: ee, debugSetBreakpoints: te, debugCommand: K, debugStop: ne } = xt({
			apiBase: m,
			defaultSource: u.code,
			persistKey: !1,
			preloadTargets: !1
		}), q = _("Output"), J = _("run"), Y = _(!1), re = [
			"Output",
			"rust",
			"c",
			"python",
			"typescript",
			"abt",
			"Bytecode"
		], ie = {
			Output: "Output",
			rust: "Rust",
			c: "C",
			python: "Python",
			typescript: "TS",
			abt: "ABT",
			Bytecode: "Bytecode"
		}, X = t(() => ({ height: u.height })), ge = t(() => q.value !== "Output" && q.value !== "Bytecode" && A.value), _e = t(() => {
			let e = q.value;
			return e !== "Output" && e !== "Bytecode" && M.value.length > 1;
		});
		function Z(e) {
			H(q.value, e);
		}
		function ve(e) {
			G(N.value, e);
		}
		async function ye() {
			J.value === "run" ? (await B(), q.value = "Output") : (V(J.value), q.value = J.value);
		}
		C(J, (e) => {
			e !== "run" && j.value && (V(e), q.value = e);
		});
		function be(e) {
			q.value = e, e !== "Output" && e !== "Bytecode" ? (J.value = e, V(e)) : e === "Output" && (J.value = "run");
		}
		function xe(e) {
			U(e), q.value = "Output", J.value = "run";
		}
		async function Se() {
			if (A.value) try {
				await navigator.clipboard.writeText(A.value), Y.value = !0, setTimeout(() => {
					Y.value = !1;
				}, 2e3);
			} catch {}
		}
		async function Ce() {
			await ee(), q.value = "Bytecode";
		}
		async function we() {
			await ne(), q.value = "Output";
		}
		function Q(e) {
			te(e);
		}
		function $(e) {}
		return (t, l) => (g(), i(e, null, [a("div", {
			class: "playground-wrapper",
			style: p(X.value)
		}, [a("div", St, [a("div", Ct, [
			c(b(le), { size: 16 }),
			l[8] ||= a("span", { class: "toolbar-title" }, "Auto Playground", -1),
			c(ut, {
				"api-base": b(m),
				onSelect: xe
			}, null, 8, ["api-base"])
		]), a("div", wt, [
			w(a("select", {
				"onUpdate:modelValue": l[0] ||= (e) => J.value = e,
				class: "target-select",
				disabled: b(z)
			}, [...l[9] ||= [o("<option value=\"run\" data-v-2aaf27a1>Run</option><option value=\"rust\" data-v-2aaf27a1>→ Rust</option><option value=\"c\" data-v-2aaf27a1>→ C</option><option value=\"python\" data-v-2aaf27a1>→ Python</option><option value=\"typescript\" data-v-2aaf27a1>→ TypeScript</option><option value=\"abt\" data-v-2aaf27a1>→ ABT</option>", 6)]], 8, Tt), [[S, J.value]]),
			b(z) ? (g(), i("div", Dt, [
				a("button", {
					class: "debug-btn continue",
					onClick: l[1] ||= (e) => b(K)("continue"),
					disabled: b(k),
					title: "Continue"
				}, [c(b(fe), { size: 14 })], 8, Ot),
				a("button", {
					class: "debug-btn step",
					onClick: l[2] ||= (e) => b(K)("step"),
					disabled: b(k),
					title: "Step Into"
				}, [c(b(ae), { size: 14 })], 8, kt),
				a("button", {
					class: "debug-btn step-over",
					onClick: l[3] ||= (e) => b(K)("step_over"),
					disabled: b(k),
					title: "Step Over"
				}, [c(b(me), { size: 14 })], 8, At),
				a("button", {
					class: "debug-btn step-out",
					onClick: l[4] ||= (e) => b(K)("step_out"),
					disabled: b(k),
					title: "Step Out"
				}, [c(b(oe), { size: 14 })], 8, jt)
			])) : (g(), i("button", {
				key: 0,
				class: "run-btn",
				onClick: ye,
				disabled: b(k)
			}, [b(k) ? (g(), n(b(de), {
				key: 1,
				size: 14,
				class: "spin"
			})) : (g(), n(b(fe), {
				key: 0,
				size: 14
			})), s(" " + y(b(k) ? "Running..." : "Run"), 1)], 8, Et)),
			b(z) ? (g(), i("button", {
				key: 2,
				class: "stop-btn",
				onClick: we,
				title: "Stop Debug"
			}, [c(b(he), { size: 14 }), l[10] ||= s(" Stop ", -1)])) : (g(), i("button", {
				key: 3,
				class: "debug-start-btn",
				onClick: Ce,
				disabled: b(k),
				title: "Start Debug"
			}, [c(b(se), { size: 14 }), l[11] ||= s(" Debug ", -1)], 8, Mt)),
			b(z) ? r("", !0) : (g(), i("label", Nt, [l[13] ||= a("span", { class: "switch-label" }, "Live", -1), a("span", Pt, [w(a("input", {
				type: "checkbox",
				"onUpdate:modelValue": l[5] ||= (e) => d(j) ? j.value = e : null
			}, null, 512), [[x, b(j)]]), l[12] ||= a("span", { class: "slider" }, null, -1)])])),
			a("button", {
				class: "icon-btn share-btn",
				onClick: l[6] ||= (...e) => b(W) && b(W)(...e),
				title: "Copy shareable link"
			}, [c(b(pe), { size: 14 })])
		])]), a("div", Ft, [a("div", It, [c(Te, {
			"model-value": b(h),
			"onUpdate:modelValue": l[7] ||= (e) => h.value = e,
			"on-run": ye,
			"is-debugging": b(z),
			breakpoints: b(R),
			"current-debug-line": b(I)?.line ?? null,
			onBreakpointsChange: Q
		}, null, 8, [
			"model-value",
			"is-debugging",
			"breakpoints",
			"current-debug-line"
		])]), a("div", Lt, [
			a("div", Rt, [
				(g(), i(e, null, v(re, (e) => a("button", {
					key: e,
					class: f(["tab-btn", { active: q.value === e }]),
					onClick: (t) => be(e)
				}, y(ie[e]), 11, zt)), 64)),
				l[14] ||= a("div", { class: "spacer" }, null, -1),
				b(z) && b(I) ? (g(), i("span", {
					key: 0,
					class: f(["debug-status", b(I).status])
				}, y(b(I).status), 3)) : r("", !0),
				ge.value ? (g(), i("button", {
					key: 1,
					class: "icon-btn copy-btn",
					onClick: Se,
					title: Y.value ? "Copied!" : "Copy code"
				}, [Y.value ? (g(), n(b(ce), {
					key: 1,
					size: 14
				})) : (g(), n(b(ue), {
					key: 0,
					size: 14
				}))], 8, Bt)) : r("", !0)
			]),
			a("div", Vt, [q.value === "Output" ? (g(), n($e, {
				key: 0,
				stdout: b(T),
				stderr: b(E),
				result: b(D),
				"time-ms": b(O)
			}, null, 8, [
				"stdout",
				"stderr",
				"result",
				"time-ms"
			])) : q.value === "Bytecode" ? (g(), n(at, {
				key: 1,
				bytecode: b(L),
				"current-ip": b(I)?.ip,
				onOffsetClick: $
			}, null, 8, ["bytecode", "current-ip"])) : (g(), i("div", Ht, [_e.value ? (g(), n(mt, {
				key: 0,
				files: b(M),
				selected: b(N),
				onSelect: Z
			}, null, 8, ["files", "selected"])) : r("", !0), c(Ke, {
				code: b(A),
				language: q.value,
				"highlight-lines": b(P),
				onLineClick: ve
			}, null, 8, [
				"code",
				"language",
				"highlight-lines"
			])]))]),
			b(z) && b(I) ? (g(), i("div", Ut, [
				b(I).stack.length ? (g(), i("div", Wt, [a("div", Gt, "Stack (" + y(b(I).stack.length) + ")", 1), a("div", Kt, [(g(!0), i(e, null, v(b(I).stack.slice(-8), (e, t) => (g(), i("span", {
					key: t,
					class: "stack-item"
				}, y(e), 1))), 128))])])) : r("", !0),
				b(I).call_stack.length ? (g(), i("div", qt, [l[15] ||= a("div", { class: "debug-section-title" }, "Call Stack", -1), (g(!0), i(e, null, v(b(I).call_stack, (e, t) => (g(), i("div", {
					key: t,
					class: "call-frame"
				}, [a("span", Jt, y(e.fn_name || "<root>"), 1), a("span", Yt, "line " + y(e.line) + ", bp=" + y(e.bp), 1)]))), 128))])) : r("", !0),
				b(I).locals.length ? (g(), i("div", Xt, [l[16] ||= a("div", { class: "debug-section-title" }, "Locals", -1), a("div", Zt, [(g(!0), i(e, null, v(b(I).locals, (e, t) => (g(), i("span", {
					key: t,
					class: "local-item"
				}, [a("span", Qt, "[" + y(e.index) + "]", 1), s(" " + y(e.value), 1)]))), 128))])])) : r("", !0),
				b(I).registers ? (g(), i("div", $t, " IP=" + y(b(I).registers.ip) + " BP=" + y(b(I).registers.bp) + " SP=" + y(b(I).registers.sp), 1)) : r("", !0)
			])) : r("", !0)
		])])], 4), a("div", { class: f(["toast", { visible: b(F).visible }]) }, y(b(F).message), 3)], 64));
	}
}), [["__scopeId", "data-v-2aaf27a1"]]), tn = { class: "debug-toolbar" }, nn = [
	"disabled",
	"onClick",
	"title"
], rn = { class: "icon" }, an = { class: "label" }, on = ["title"], sn = { class: "icon" }, cn = { class: "label" }, ln = ["disabled"], un = /* @__PURE__ */ Q(/* @__PURE__ */ l({
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
				title: "F5"
			},
			{
				cmd: "step",
				icon: "↓",
				label: "Step Into",
				title: "F11"
			},
			{
				cmd: "step_over",
				icon: "→",
				label: "Step Over",
				title: "F10"
			},
			{
				cmd: "step_out",
				icon: "↑",
				label: "Step Out",
				title: "Shift+F11"
			}
		];
		return (r, o) => (g(), i("div", tn, [
			(g(), i(e, null, v(n, (e) => a("button", {
				key: e.cmd,
				disabled: !t.isPaused,
				onClick: (t) => r.$emit("command", e.cmd),
				title: e.title
			}, [a("span", rn, y(e.icon), 1), a("span", an, y(e.label), 1)], 8, nn)), 64)),
			a("button", {
				class: "stop-btn",
				onClick: o[0] ||= (e) => r.$emit("command", "stop"),
				title: "Stop Debugging"
			}, [...o[3] ||= [a("span", { class: "icon" }, "■", -1), a("span", { class: "label" }, "Stop", -1)]]),
			o[5] ||= a("div", { class: "toolbar-divider" }, null, -1),
			a("button", {
				class: f(["record-btn", { recording: t.isRecording }]),
				onClick: o[1] ||= (e) => r.$emit("toggleRecord"),
				title: t.isRecording ? "Stop Recording" : "Start Recording"
			}, [a("span", sn, y(t.isRecording ? "⏹" : "⏺"), 1), a("span", cn, y(t.isRecording ? "Recording" : "Record"), 1)], 10, on),
			a("button", {
				class: "save-btn",
				onClick: o[2] ||= (e) => r.$emit("exportRecording"),
				disabled: !t.hasRecording,
				title: "Export Replay File"
			}, [...o[4] ||= [a("span", { class: "icon" }, "💾", -1), a("span", { class: "label" }, "Save", -1)]], 8, ln)
		]));
	}
}), [["__scopeId", "data-v-1898df5a"]]), dn = { class: "replay-toolbar" }, fn = ["title"], pn = { class: "icon" }, mn = { class: "timeline" }, hn = ["max", "value"], gn = { class: "frame-info" }, _n = /* @__PURE__ */ Q(/* @__PURE__ */ l({
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
		return (t, n) => (g(), i("div", dn, [
			a("button", {
				onClick: n[0] ||= (n) => e.isPlaying ? t.$emit("pause") : t.$emit("play"),
				title: e.isPlaying ? "Pause" : "Play"
			}, [a("span", pn, y(e.isPlaying ? "⏸" : "▶"), 1)], 8, fn),
			a("button", {
				onClick: n[1] ||= (e) => t.$emit("stepBackward"),
				title: "Step Backward (←)"
			}, [...n[3] ||= [a("span", { class: "icon" }, "⏮", -1)]]),
			a("button", {
				onClick: n[2] ||= (e) => t.$emit("stepForward"),
				title: "Step Forward (→)"
			}, [...n[4] ||= [a("span", { class: "icon" }, "⏭", -1)]]),
			a("div", mn, [a("input", {
				type: "range",
				min: 0,
				max: Math.max(0, s.value - 1),
				value: o.value,
				onInput: l,
				class: "timeline-slider"
			}, null, 40, hn), a("span", gn, "Frame " + y(o.value + 1) + " / " + y(s.value), 1)]),
			n[5] ||= a("div", { class: "replay-badge" }, "🔁 Replay Mode", -1)
		]));
	}
}), [["__scopeId", "data-v-1b1084c5"]]), vn = { class: "debug-aux-panel" }, yn = { class: "aux-section" }, bn = {
	key: 0,
	class: "var-group"
}, xn = { class: "var-name" }, Sn = { class: "var-value" }, Cn = {
	key: 1,
	class: "var-group"
}, wn = { class: "var-name" }, Tn = { class: "var-value" }, En = {
	key: 2,
	class: "var-group"
}, Dn = { class: "var-name" }, On = { class: "var-value" }, kn = {
	key: 3,
	class: "empty"
}, An = { class: "aux-section" }, jn = { class: "callstack-list" }, Mn = { class: "cs-name" }, Nn = { class: "cs-line" }, Pn = {
	key: 0,
	class: "empty"
}, Fn = { class: "aux-section compact" }, In = { class: "reg-row" }, Ln = { class: "reg-value" }, Rn = { class: "reg-row" }, zn = { class: "reg-value" }, Bn = { class: "reg-row" }, Vn = { class: "reg-value" }, Hn = {
	key: 0,
	class: "aux-section stdout-section"
}, Un = { class: "stdout-content" }, Wn = /* @__PURE__ */ Q(/* @__PURE__ */ l({
	__name: "DebugAuxPanel",
	props: { state: {} },
	setup(n) {
		let o = n, s = _(!1), c = _(!1);
		C(() => o.state?.stack, (e, t) => {
			let n = e || [], r = t || [];
			n.length > r.length ? s.value = !0 : n.length < r.length && (c.value = !0), setTimeout(() => {
				s.value = !1, c.value = !1;
			}, 600);
		}, {
			deep: !0,
			flush: "post"
		});
		let l = t(() => {
			if (!o.state) return [];
			let e = o.state.stack, t = Math.min(e.length, 8);
			return [...e.slice(-t)].reverse().map((e, t) => ({
				value: e,
				distFromTop: t
			}));
		}), u = t(() => {
			if (!o.state) return [];
			let e = [...o.state.call_stack];
			return e.push({
				fn_name: null,
				line: o.state.line,
				return_ip: o.state.registers.ip,
				bp: o.state.registers.bp,
				n_args: o.state.args?.length ?? 0,
				n_locals: o.state.locals?.length ?? 0
			}), e.reverse();
		}), d = t(() => (o.state?.args?.length ?? 0) > 0 || (o.state?.locals?.length ?? 0) > 0 || l.value.length > 0);
		function p(e) {
			return e === void 0 ? "-" : `0x${e.toString(16).padStart(4, "0")}`;
		}
		return (t, m) => (g(), i("div", vn, [
			a("div", yn, [
				m[3] ||= a("div", { class: "aux-title" }, "Variables", -1),
				o.state?.args?.length ? (g(), i("div", bn, [m[0] ||= a("div", { class: "var-group-title" }, "Arguments", -1), (g(!0), i(e, null, v(o.state.args, (e) => (g(), i("div", {
					key: "arg" + e.index,
					class: "var-row"
				}, [a("span", xn, "arg" + y(e.index), 1), a("span", Sn, y(e.value), 1)]))), 128))])) : r("", !0),
				o.state?.locals?.length ? (g(), i("div", Cn, [m[1] ||= a("div", { class: "var-group-title" }, "Locals", -1), (g(!0), i(e, null, v(o.state.locals, (e) => (g(), i("div", {
					key: "loc" + e.index,
					class: "var-row"
				}, [a("span", wn, "local" + y(e.index), 1), a("span", Tn, y(e.value), 1)]))), 128))])) : r("", !0),
				l.value.length ? (g(), i("div", En, [m[2] ||= a("div", { class: "var-group-title" }, "Stack Top", -1), (g(!0), i(e, null, v(l.value, (e, t) => (g(), i("div", {
					key: "stk" + t,
					class: f(["var-row", {
						"is-top": t === 0,
						"is-pushed": s.value && t === 0,
						"is-popped": c.value && t === 0
					}])
				}, [a("span", Dn, "[" + y(e.distFromTop) + "]", 1), a("span", On, y(e.value), 1)], 2))), 128))])) : r("", !0),
				d.value ? r("", !0) : (g(), i("div", kn, "No variables"))
			]),
			a("div", An, [
				m[4] ||= a("div", { class: "aux-title" }, "Call Stack", -1),
				a("div", jn, [(g(!0), i(e, null, v(u.value, (e, t) => (g(), i("div", {
					key: t,
					class: f(["callstack-item", { "is-current": t === 0 }])
				}, [a("span", Mn, y(e.fn_name ?? "<main>"), 1), a("span", Nn, ":" + y(e.line), 1)], 2))), 128))]),
				o.state?.call_stack?.length ? r("", !0) : (g(), i("div", Pn, "No frames"))
			]),
			a("div", Fn, [
				m[8] ||= a("div", { class: "aux-title" }, "Registers", -1),
				a("div", In, [m[5] ||= a("span", { class: "reg-label" }, "IP", -1), a("span", Ln, y(p(n.state?.registers.ip)), 1)]),
				a("div", Rn, [m[6] ||= a("span", { class: "reg-label" }, "BP", -1), a("span", zn, y(p(n.state?.registers.bp)), 1)]),
				a("div", Bn, [m[7] ||= a("span", { class: "reg-label" }, "SP", -1), a("span", Vn, y(p(n.state?.registers.sp)), 1)])
			]),
			n.state?.stdout ? (g(), i("div", Hn, [m[9] ||= a("div", { class: "aux-title" }, "Output", -1), a("pre", Un, y(n.state.stdout), 1)])) : r("", !0)
		]));
	}
}), [["__scopeId", "data-v-21ab0f4b"]]), Gn = { class: "playground" }, Kn = { class: "toolbar" }, qn = { class: "toolbar-left" }, Jn = { class: "toolbar-right" }, Yn = ["disabled"], Xn = ["disabled"], Zn = ["disabled"], Qn = ["disabled"], $n = { class: "trans-current" }, er = { class: "workspace" }, tr = { class: "main-row" }, nr = { class: "pane-header" }, rr = { key: 0 }, ir = { key: 1 }, ar = {
	key: 0,
	class: "active-file-name"
}, or = { class: "pane-body" }, sr = {
	key: 0,
	class: "preview-pane"
}, cr = { class: "pane-header" }, lr = ["disabled"], ur = {
	key: 0,
	class: "output-pane"
}, dr = { class: "pane-header" }, fr = { class: "output-body" }, pr = /* @__PURE__ */ Q(/* @__PURE__ */ l({
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
		let d = l, h = u, v = t({
			get: () => d.transTarget,
			set: (e) => h("update:transTarget", e)
		}), b = t(() => ({
			rust: "Rust",
			c: "C",
			python: "Python",
			typescript: "TypeScript",
			abt: "ABT",
			bytecode: "Bytecode"
		})[d.transTarget] ?? d.transTarget), x = _(null), T = _("auto");
		m(async () => {
			await document.fonts?.ready;
			let e = x.value;
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
			T.value = `${Math.ceil(i + 12 + 20 + 4)}px`;
		});
		let E = _(!1);
		C(() => d.mode, () => {
			E.value = !1;
		});
		let D = t(() => {
			let e = d.transTarget;
			return d.mode === "trans" && (e === "python" || e === "typescript");
		});
		async function O() {
			let e = d.transTarget;
			!e || !d.onRunCode || (E.value = !0, await d.onRunCode(e));
		}
		let k = t(() => d.mode === "run" || d.mode === "debug" || d.mode === "replay" ? "Bytecode" : d.mode === "trans" ? b.value : ""), A = t(() => {
			if (d.mode === "trans") return d.transTarget;
		}), j = t(() => d.mode === "trans" && (d.transFiles?.length ?? 0) > 1), M = t(() => (d.projectFiles?.length ?? 0) > 1), N = t(() => d.mappedSourceFiles ? Array.from(d.mappedSourceFiles) : []);
		function P(e) {
			d.onOutputLineClick?.(d.selectedTransFile ?? "", e);
		}
		let F = t(() => (d.mode, d.bytecode ?? []));
		function I(e) {
			d.onSelectTransFile?.(d.transTarget, e);
		}
		let L = t(() => d.mode === "run" || d.mode === "debug" || d.mode === "replay" || E.value), R = t(() => d.mode === "run" || E.value ? "Output" : d.mode === "debug" || d.mode === "replay" ? "Debug Output" : "");
		function z() {
			d.onTrans();
		}
		function B(e) {
			h("loadExample", e);
		}
		return (t, u) => (g(), i("div", Gn, [
			a("header", Kn, [a("div", qn, [u[21] ||= a("h1", { class: "title" }, "Auto Playground", -1), c(ut, { onSelect: B })]), a("div", Jn, [
				!l.isDebugging && !l.isReplayMode ? (g(), i("button", {
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
					class: f(["toolbar-btn debug-btn", { active: l.isDebugging }]),
					onClick: u[2] ||= (...e) => d.onDebug && d.onDebug(...e),
					disabled: l.isLoading || l.isReplayMode,
					title: "Start Debugging"
				}, [...u[24] ||= [o("<svg width=\"14\" height=\"14\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2.5\" stroke-linecap=\"round\" stroke-linejoin=\"round\" data-v-d05c2fa5><path d=\"M12 2a10 10 0 0 1 10 10\" data-v-d05c2fa5></path><path d=\"M12 2a10 10 0 0 0-10 10\" data-v-d05c2fa5></path><path d=\"M12 12l4-4\" data-v-d05c2fa5></path><path d=\"M12 12l-4-4\" data-v-d05c2fa5></path><path d=\"M12 12l4 4\" data-v-d05c2fa5></path><path d=\"M12 12l-4 4\" data-v-d05c2fa5></path></svg> Debug ", 2)]], 10, Yn),
				a("button", {
					class: "toolbar-btn run-btn",
					onClick: u[3] ||= (...e) => d.onRun && d.onRun(...e),
					disabled: l.isLoading || l.isReplayMode
				}, y(l.isLoading ? "Running..." : "Run (Ctrl+Enter)"), 9, Xn),
				a("div", {
					class: f(["trans-split-btn", { disabled: l.isLoading || l.isReplayMode }]),
					title: "Transpile to target language"
				}, [a("button", {
					class: "trans-main",
					onClick: u[4] ||= (...e) => d.onTrans && d.onTrans(...e),
					disabled: l.isLoading || l.isReplayMode
				}, " Trans ", 8, Zn), a("div", {
					ref_key: "transDropdownEl",
					ref: x,
					class: "trans-dropdown",
					style: p({ width: T.value })
				}, [
					w(a("select", {
						"onUpdate:modelValue": u[5] ||= (e) => v.value = e,
						class: "trans-select",
						disabled: l.isLoading || l.isDebugging || l.isReplayMode,
						onChange: z
					}, [...u[25] ||= [o("<option value=\"rust\" data-v-d05c2fa5>Rust</option><option value=\"c\" data-v-d05c2fa5>C</option><option value=\"python\" data-v-d05c2fa5>Python</option><option value=\"typescript\" data-v-d05c2fa5>TypeScript</option><option value=\"abt\" data-v-d05c2fa5>ABT</option>", 5)]], 40, Qn), [[S, v.value]]),
					a("span", $n, y(b.value), 1),
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
				], 4)], 2)
			])]),
			l.isDebugging || l.hasRecording ? (g(), n(un, {
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
			l.isReplayMode ? (g(), n(_n, {
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
			a("div", er, [a("div", tr, [a("div", { class: f(["editor-pane", { "with-preview": l.mode !== "editor" }]) }, [a("div", nr, [l.isReplayMode ? (g(), i("span", rr, "Replay")) : (g(), i("span", ir, [u[27] ||= s("Auto ", -1), l.activeFile ? (g(), i("span", ar, "· " + y(l.activeFile), 1)) : r("", !0)]))]), a("div", or, [M.value ? (g(), n(mt, {
				key: 0,
				files: l.projectFiles,
				selected: l.activeFile || "",
				"mapped-files": N.value,
				onSelect: u[14] ||= (e) => t.$emit("selectFile", e)
			}, null, 8, [
				"files",
				"selected",
				"mapped-files"
			])) : r("", !0), c(Te, {
				"model-value": l.source,
				"onUpdate:modelValue": u[15] ||= (e) => t.$emit("update:source", e),
				"on-run": l.onRun,
				"is-debugging": l.isDebugging || l.isReplayMode,
				breakpoints: l.breakpoints,
				"current-debug-line": l.currentDebugLine,
				"highlighted-source-line": l.currentSourceLine,
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
				"read-only"
			])])], 2), l.mode === "editor" ? r("", !0) : (g(), i("div", sr, [a("div", cr, [a("span", null, y(k.value), 1), D.value ? (g(), i("button", {
				key: 0,
				class: "run-code-btn",
				disabled: l.isLoading || l.isReplayMode,
				onClick: O
			}, " Run " + y(b.value), 9, lr)) : r("", !0)]), a("div", { class: f(["pane-body", { "with-file-tree": j.value }]) }, [l.mode === "run" || l.mode === "debug" || l.mode === "replay" ? (g(), n(at, {
				key: 0,
				bytecode: F.value,
				"current-ip": l.debugState?.ip,
				"highlighted-offsets": l.highlightedOffsets,
				onOffsetClick: u[20] ||= (e) => t.$emit("offsetClick", e)
			}, null, 8, [
				"bytecode",
				"current-ip",
				"highlighted-offsets"
			])) : l.mode === "trans" ? (g(), i(e, { key: 1 }, [j.value ? (g(), n(mt, {
				key: 0,
				files: l.transFiles || [],
				selected: l.selectedTransFile || "",
				onSelect: I
			}, null, 8, ["files", "selected"])) : r("", !0), c(Ke, {
				code: l.transpiledCode,
				language: A.value,
				"highlight-lines": l.highlightLines,
				onLineClick: P
			}, null, 8, [
				"code",
				"language",
				"highlight-lines"
			])], 64)) : r("", !0)], 2)]))]), L.value ? (g(), i("div", ur, [a("div", dr, [a("span", null, y(R.value), 1)]), a("div", fr, [c($e, {
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
			]), (l.isDebugging || l.isReplayMode) && l.debugState ? (g(), n(Wn, {
				key: 0,
				state: l.debugState
			}, null, 8, ["state"])) : r("", !0)])])) : r("", !0)])
		]));
	}
}), [["__scopeId", "data-v-d05c2fa5"]]), mr = "/api", hr = "auto-playground:state", gr = "// Welcome to Auto Playground!\nfn add(a int, b int) int {\n    a + b\n}\n\nlet result = add(3, 4)\nprint(result)";
function _r() {
	let e = window.location.hash;
	if (e.startsWith("#share=")) try {
		let t = atob(decodeURIComponent(e.slice(7))), n = JSON.parse(t);
		if (n.source) return n;
	} catch {}
	try {
		let e = localStorage.getItem(hr);
		if (e) return JSON.parse(e);
	} catch {}
	return {};
}
function vr(e) {
	try {
		localStorage.setItem(hr, JSON.stringify(e));
	} catch {}
}
function yr() {
	let e = _r(), n = _(e.source ?? gr), r = _(""), i = _(""), a = _(""), o = _(0), s = _([]), c = _(!1), l = _(e.activeTab ?? "rust"), u = _(""), d = _(e.projectDir), f = _(e.projectFiles ?? []), p = _(e.activeFile ?? "");
	function m() {
		if (!p.value) return;
		let e = f.value.find((e) => e.path === p.value);
		e && (e.source = n.value);
	}
	function h(e) {
		if (e === p.value) return;
		m(), p.value = e;
		let t = f.value.find((t) => t.path === e);
		t && (n.value = t.source);
	}
	function g(e) {
		if (d.value) {
			m(), e.project_dir = d.value, e.files = f.value;
			let t = f.value.find((e) => e.path === "main.at");
			t && (e.source = t.source);
		}
		return e;
	}
	let v = _({}), y = _(null), b = _([]), x = _([]), S = _({
		message: "",
		visible: !1
	}), w = t(() => {
		let e = u.value;
		if (!e) return "";
		let t = v.value[e];
		return t ? t.files.find((e) => e.path === t.selectedFile)?.code ?? t.files[0]?.code ?? "" : "";
	}), T = t(() => {
		let e = u.value;
		return e ? v.value[e]?.files ?? [] : [];
	}), E = t(() => {
		let e = u.value;
		return e ? v.value[e]?.selectedFile ?? "" : "";
	}), D = t(() => p.value || ""), O = t(() => {
		let e = u.value, t = /* @__PURE__ */ new Map();
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
	}), k = t(() => {
		let e = u.value, t = /* @__PURE__ */ new Map();
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
	}), A = t(() => {
		let e = u.value, t = /* @__PURE__ */ new Set();
		if (!e) return t;
		let n = v.value[e];
		if (!n) return t;
		for (let e of n.files) for (let r of n.fileSourceMaps[e.path] ?? []) t.add(r.source_file || D.value);
		return t;
	});
	function j() {
		y.value ? M(y.value) : (b.value = [], x.value = []);
	}
	function M(e) {
		y.value = e;
		let t = D.value, n = O.value.get(t)?.get(e) ?? [];
		x.value = n.map((e) => e.outputFile);
		let r = E.value;
		b.value = n.find((e) => e.outputFile === r)?.outputLines ?? [];
	}
	function N(e, t) {
		let n = k.value.get(e)?.get(t);
		if (!n) {
			F();
			return;
		}
		n.sourceFile && f.value.length > 0 && p.value !== n.sourceFile && h(n.sourceFile), y.value = n.sourceLine;
		let r = O.value.get(n.sourceFile)?.get(n.sourceLine) ?? [];
		x.value = r.map((e) => e.outputFile), b.value = r.find((t) => t.outputFile === e)?.outputLines ?? [];
	}
	function P(e, t) {
		return k.value.get(e)?.get(t)?.sourceFile;
	}
	function F() {
		y.value = null, b.value = [], x.value = [];
	}
	async function I() {
		c.value = !0, r.value = "", i.value = "", a.value = "", s.value = [];
		try {
			let e = g({ source: n.value }), t = await (await fetch(`${mr}/run`, {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify(e)
			})).json();
			r.value = t.stdout || "", i.value = t.stderr || "", o.value = t.time_ms || 0, s.value = t.bytecode || [], t.result !== void 0 && t.result !== null && t.result !== "" && (a.value = t.result);
		} catch (e) {
			i.value = `Network error: ${e.message}`;
		} finally {
			c.value = !1;
		}
	}
	async function L(e) {
		c.value = !0, r.value = "", i.value = "", a.value = "", o.value = 0;
		let t = v.value[e]?.files[0]?.code ?? "";
		if (!t.trim()) {
			i.value = `No ${e} code to run. Make sure the transpilation succeeded.`, c.value = !1;
			return;
		}
		try {
			if (e === "typescript") {
				let e = await vt(t);
				r.value = e.stdout, i.value = e.stderr, o.value = 0;
			} else {
				let n = await (await fetch(`${mr}/run_code`, {
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
			c.value = !1;
		}
	}
	async function R(e) {
		c.value = !0;
		try {
			let t = g({
				source: n.value,
				target: e
			}), r = await (await fetch(`${mr}/trans`, {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify(t)
			})).json(), i = r.files ?? [], a = {};
			for (let e of i) a[e.path] = e.source_map ?? r.source_map ?? [];
			let o = i[0]?.path ?? "";
			u.value = e, v.value[e] = {
				files: i,
				fileSourceMaps: a,
				selectedFile: o
			}, j();
		} catch (t) {
			u.value = e, v.value[e] = {
				files: [{
					path: "error.txt",
					code: `Error: ${t.message}`
				}],
				fileSourceMaps: { "error.txt": [] },
				selectedFile: "error.txt"
			}, j();
		} finally {
			c.value = !1;
		}
	}
	function z(e) {
		l.value = e, u.value = e, j();
	}
	function B(e, t) {
		let n = v.value[e];
		n && (n.selectedFile = t, j());
	}
	function V(e) {
		n.value = e.source, d.value = e.project_dir, f.value = e.files ?? [], p.value = e.files?.length ? "main.at" : "", r.value = "", i.value = "", a.value = "", s.value = [], y.value = null, b.value = [], x.value = [];
	}
	function H() {
		let e = JSON.stringify({
			source: n.value,
			activeTab: l.value,
			projectDir: d.value,
			projectFiles: f.value.length ? f.value : void 0,
			activeFile: p.value || void 0
		}), t = "#share=" + encodeURIComponent(btoa(e));
		return window.location.origin + window.location.pathname + t;
	}
	async function U() {
		let e = H(), t = !1;
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
		S.value = {
			message: t ? "Share link copied to clipboard!" : "Failed to copy link",
			visible: !0
		}, setTimeout(() => {
			S.value.visible = !1;
		}, 2500);
	}
	return C(n, () => {
		v.value = {}, u.value && (u.value = "", b.value = [], x.value = []);
	}), C([
		n,
		l,
		d,
		f,
		p
	], ([e, t, n, r, i]) => {
		vr({
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
		isLoading: c,
		activeTab: l,
		transpiledCode: w,
		transpileTarget: u,
		projectDir: d,
		projectFiles: f,
		activeFile: p,
		transFiles: T,
		selectedTransFile: E,
		highlightedSourceLine: y,
		highlightedOutputLines: b,
		highlightedOutputFiles: x,
		mappedSourceFiles: A,
		shareToast: S,
		run: I,
		runCode: L,
		transpile: R,
		switchTab: z,
		selectTransFile: B,
		selectFile: h,
		loadExample: V,
		highlightSourceLine: M,
		highlightOutputLine: N,
		getSourceFileForOutputLine: P,
		clearHighlight: F,
		share: U
	};
}
//#endregion
//#region src/composables/useDebugger.ts
function br() {
	let e = _(null), n = _(!1), r = _(!1), i = _([]), a = _(null), o = _(null), s = _(!1), c = _(null), l = t(() => {
		let e = {};
		for (let t of i.value) t.line !== void 0 && (e[t.line] || (e[t.line] = []), e[t.line].push(t.offset));
		return e;
	}), u = t(() => {
		let e = {};
		for (let t of i.value) t.line !== void 0 && (e[t.offset] = t.line);
		return e;
	});
	function d(t, i = []) {
		if (e.value) return;
		let a = window.location.protocol === "https:" ? "wss:" : "ws:", s = new WebSocket(`${a}//${window.location.host}/api/debug/ws`);
		s.onopen = () => {
			n.value = !0, r.value = !0, s.send(JSON.stringify({
				type: "debug.start",
				source: t
			})), i.length > 0 && s.send(JSON.stringify({
				type: "breakpoints.set",
				lines: i
			}));
		}, s.onmessage = (e) => {
			f(JSON.parse(e.data));
		}, s.onerror = (e) => {
			o.value = "WebSocket error", console.error("Debug WS error:", e);
		}, s.onclose = () => {
			n.value = !1, r.value = !1, e.value = null;
		}, e.value = s;
	}
	function f(e) {
		switch (e.type) {
			case "bytecode":
				i.value = e.lines || [], s.value && c.value && (c.value.bytecode = e.lines || []);
				break;
			case "state":
				a.value = e.data, s.value && c.value && c.value.events.push({
					type: "state",
					state: e.data
				}), (e.data.status === "finished" || e.data.status === "error") && (r.value = !1);
				break;
			case "error":
				o.value = e.message, r.value = !1;
				break;
		}
	}
	function p(t) {
		e.value?.readyState === WebSocket.OPEN && e.value.send(JSON.stringify({
			type: "command",
			cmd: t
		})), s.value && c.value && c.value.events.push({
			type: "command",
			cmd: t
		});
	}
	function m(t) {
		e.value?.readyState === WebSocket.OPEN && e.value.send(JSON.stringify({
			type: "breakpoints.set",
			lines: t
		})), s.value && c.value && c.value.events.push({
			type: "breakpoints",
			lines: t
		});
	}
	function h() {
		p("stop"), e.value?.close(), e.value = null, r.value = !1, a.value = null, i.value = [], o.value = null;
	}
	function g(e, t) {
		c.value = {
			version: 1,
			createdAt: (/* @__PURE__ */ new Date()).toISOString(),
			source: e,
			initialBreakpoints: [...t],
			bytecode: [],
			events: []
		}, s.value = !0;
	}
	function v() {
		return s.value = !1, c.value;
	}
	function y() {
		if (!c.value) return;
		let e = new Blob([JSON.stringify(c.value, null, 2)], { type: "application/json" }), t = URL.createObjectURL(e), n = document.createElement("a");
		n.href = t, n.download = `replay_${Date.now()}.autoreplay`, n.click(), URL.revokeObjectURL(t);
	}
	return {
		isConnected: n,
		isDebugging: r,
		bytecode: i,
		state: a,
		error: o,
		lineToOffsets: l,
		offsetToLine: u,
		connect: d,
		sendCommand: p,
		setBreakpoints: m,
		stop: h,
		isRecording: s,
		recording: c,
		startRecording: g,
		stopRecording: v,
		exportRecording: y
	};
}
//#endregion
//#region src/composables/useReplayPlayer.ts
function xr() {
	let e = _(!1), n = _(null), r = _(0), i = _(!1), a = null, o = t(() => n.value?.events ?? []), s = t(() => o.value.filter((e) => e.type === "state").map((e, t) => ({
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
	}), u = t(() => n.value?.bytecode ?? []), d = t(() => {
		let e = {};
		for (let t of u.value) t.line !== void 0 && (e[t.line] || (e[t.line] = []), e[t.line].push(t.offset));
		return e;
	}), f = t(() => {
		let e = {};
		for (let t of u.value) t.line !== void 0 && (e[t.offset] = t.line);
		return e;
	});
	function p(t) {
		m(), n.value = t, e.value = !0, r.value = 0;
	}
	function m() {
		g(), e.value = !1, n.value = null, r.value = 0;
	}
	function h() {
		i.value || (i.value = !0, a = setInterval(() => {
			if (r.value >= o.value.length - 1) {
				g();
				return;
			}
			r.value++;
		}, 800));
	}
	function g() {
		i.value = !1, a &&= (clearInterval(a), null);
	}
	function v() {
		g(), r.value < o.value.length - 1 && r.value++;
	}
	function y() {
		g(), r.value > 0 && r.value--;
	}
	function b(e) {
		g(), r.value = Math.max(0, Math.min(o.value.length - 1, e));
	}
	return {
		isActive: e,
		recording: n,
		currentIndex: r,
		isPlaying: i,
		currentState: l,
		bytecode: u,
		lineToOffsets: d,
		offsetToLine: f,
		totalFrames: c,
		load: p,
		stop: m,
		play: h,
		pause: g,
		stepForward: v,
		stepBackward: y,
		seek: b
	};
}
//#endregion
//#region src/AutoPlaygroundFull.vue
var Sr = /* @__PURE__ */ l({
	__name: "AutoPlaygroundFull",
	setup(n) {
		let { source: r, stdout: o, stderr: s, resultCode: l, timeMs: u, bytecode: d, isLoading: p, activeTab: v, transpiledCode: x, transFiles: S, selectedTransFile: w, projectFiles: T, activeFile: E, highlightedOutputLines: D, highlightedSourceLine: O, mappedSourceFiles: k, run: A, transpile: j, runCode: M, selectTransFile: N, selectFile: P, loadExample: F, highlightSourceLine: I, highlightOutputLine: L, clearHighlight: R, share: z, shareToast: B } = yr(), V = br(), H = xr(), U = _([]), W = _("editor"), G = _("rust"), ee = t(() => H.isActive.value ? H.currentState.value : V.state.value), te = t(() => H.isActive.value ? H.bytecode.value : V.bytecode.value), K = t(() => W.value === "run" ? d.value : te.value), ne = t(() => {
			if (O.value) return H.isActive.value ? H.lineToOffsets.value[O.value] : V.lineToOffsets.value[O.value];
		});
		C(() => V.state.value, (e) => {
			e?.status === "finished" && (o.value = e.stdout || "", l.value = e.result || "", s.value = e.stderr || "");
		}), C(() => V.isDebugging.value, (e) => {
			!e && W.value === "debug" && (W.value = "editor");
		}), C(() => H.isActive.value, (e) => {
			!e && W.value === "replay" && (W.value = "editor");
		});
		async function q() {
			W.value = "run", o.value = "", s.value = "", l.value = "", await A();
		}
		async function J() {
			W.value = "trans", await j(G.value), v.value = G.value;
		}
		async function Y(e) {
			await M(e);
		}
		function re() {
			V.isDebugging.value || (W.value = "debug", H.stop(), V.connect(r.value, U.value));
		}
		function ie() {
			V.isRecording.value ? V.stopRecording() : V.startRecording(r.value, U.value);
		}
		function X(e) {
			V.sendCommand(e);
		}
		function ae(e) {
			let t = H.isActive.value ? H.offsetToLine.value[e] : V.offsetToLine.value[e];
			t && I(t);
		}
		function oe(e) {
			U.value = e, V.setBreakpoints(e);
		}
		async function se() {
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
		function ce(e) {
			F(e), W.value = "editor";
		}
		function le(e) {
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
					e.preventDefault(), X("continue");
					break;
				case "F10":
					e.preventDefault(), X("step_over");
					break;
				case "F11":
					e.preventDefault(), X(e.shiftKey ? "step_out" : "step");
					break;
			}
		}
		return m(() => {
			window.addEventListener("keydown", le), window.__loadReplayForTest__ = (e) => {
				H.load(e), W.value = "replay";
			};
		}), h(() => {
			window.removeEventListener("keydown", le);
		}), (t, n) => (g(), i(e, null, [c(pr, {
			source: b(r),
			"is-loading": b(p),
			mode: W.value,
			"trans-target": G.value,
			"onUpdate:transTarget": n[0] ||= (e) => G.value = e,
			stdout: b(o),
			stderr: b(s),
			"result-code": b(l),
			"time-ms": b(u),
			"transpiled-code": b(x),
			"trans-files": b(S),
			"selected-trans-file": b(w),
			"project-files": b(T),
			"active-file": b(E),
			"mapped-source-files": b(k),
			"highlight-lines": b(D),
			"on-run": q,
			"on-trans": J,
			"on-run-code": Y,
			"on-debug": re,
			"on-select-trans-file": b(N),
			"on-output-line-click": b(L),
			"is-debugging": b(V).isDebugging.value,
			"is-paused": b(V).state.value?.status === "paused",
			"is-recording": b(V).isRecording.value,
			"has-recording": !!b(V).recording.value,
			bytecode: K.value,
			"debug-state": ee.value,
			"current-source-line": b(O),
			"highlighted-offsets": ne.value,
			breakpoints: U.value,
			"current-debug-line": ee.value?.line ?? null,
			"is-replay-mode": b(H).isActive.value,
			"replay-current-index": b(H).currentIndex.value,
			"replay-total-frames": b(H).totalFrames.value,
			"is-replay-playing": b(H).isPlaying.value,
			"onUpdate:source": n[1] ||= (e) => r.value = e,
			onLoadExample: ce,
			onSelectFile: b(P),
			onShare: b(z),
			onDebugCommand: X,
			onToggleRecord: ie,
			onExportRecording: b(V).exportRecording,
			onLineClick: b(I),
			"on-highlight-line": b(I),
			"on-clear-highlight": b(R),
			onOffsetClick: ae,
			onBreakpointsChange: oe,
			onLoadReplay: se,
			onReplayPlay: b(H).play,
			onReplayPause: b(H).pause,
			onReplayStepForward: b(H).stepForward,
			onReplayStepBackward: b(H).stepBackward,
			onReplaySeek: b(H).seek
		}, null, 8, /* @__PURE__ */ "source.is-loading.mode.trans-target.stdout.stderr.result-code.time-ms.transpiled-code.trans-files.selected-trans-file.project-files.active-file.mapped-source-files.highlight-lines.on-select-trans-file.on-output-line-click.is-debugging.is-paused.is-recording.has-recording.bytecode.debug-state.current-source-line.highlighted-offsets.breakpoints.current-debug-line.is-replay-mode.replay-current-index.replay-total-frames.is-replay-playing.onSelectFile.onShare.onExportRecording.onLineClick.on-highlight-line.on-clear-highlight.onReplayPlay.onReplayPause.onReplayStepForward.onReplayStepBackward.onReplaySeek".split(".")), a("div", { class: f(["toast", { visible: b(B).visible }]) }, y(b(B).message), 3)], 64));
	}
});
//#endregion
export { en as AutoPlayground, Sr as AutoPlaygroundFull, at as BytecodePanel, Te as CodeEditor, Ke as CodePreview, $e as ConsoleOutput, ut as ExampleSelector, pr as PlaygroundLayout, xe as autoLanguage, br as useDebugger, xt as usePlayground, yr as usePlaygroundFull, xr as useReplayPlayer };
