it("es7.objectRestSpread", () => {
	let original = {
		a: 1,
		b: 2
	};
	let copy = babelHelpers.extends({}, (babelHelpers.objectDestructuringEmpty(original), original));
});
