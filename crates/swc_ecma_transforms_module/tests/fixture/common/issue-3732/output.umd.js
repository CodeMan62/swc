// index.ts
(function(global, factory) {
    if (typeof module === "object" && typeof module.exports === "object") factory(exports, require("./get"));
    else if (typeof define === "function" && define.amd) define([
        "exports",
        "./get"
    ], factory);
    else if (global = typeof globalThis !== "undefined" ? globalThis : global || self) factory(global.input = {}, global.get);
})(this, function(exports, _get) {
    "use strict";
    Object.defineProperty(exports, "__esModule", {
        value: true
    });
    function _export(target, all) {
        for(var name in all)Object.defineProperty(target, name, {
            enumerable: true,
            get: Object.getOwnPropertyDescriptor(all, name).get
        });
    }
    _export(exports, {
        get byID () {
            return byID;
        },
        get get () {
            return _get;
        }
    });
    _get = /*#__PURE__*/ _interop_require_wildcard(_get);
    const byID = (id)=>{
        // Do some async stuff
        return new Promise((resolve)=>setTimeout(()=>{
                resolve("result");
            }, 2000));
    };
});
