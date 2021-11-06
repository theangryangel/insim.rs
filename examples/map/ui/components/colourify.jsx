import React from "react";

const colours = {
  0: "text-gray-500", // this should be black really
  1: "text-red-500",
  2: "text-green-200",
  3: "text-yellow-300",
  4: "text-blue-900",
  5: "text-purple-500",
  6: "text-blue-500",
  7: "text-gray-500", // white, but not white for us
  8: "text-gray-500", // default colour
  9: "text-gray-500", // reset
};

function sanitize(string) {
  const map = {
    "&": "&amp;",
    "<": "&lt;",
    ">": "&gt;",
    '"': "&quot;",
    "'": "&#x27;",
    "/": "&#x2F;",
  };
  const reg = /[&<>"'/]/gi;
  return string.replace(reg, (match) => map[match]);
}

// Courtesy of https://apidocs.tc-gaming.co.uk/guides/converting-lfs-colours
// with minor modifications
// TODO: this is bugged.
function colourise(str) {
  var parts = str.split(/(\^\d)/g).filter((i) => { return i.length > 0; });

  var res = "";
  parts.forEach(function (el, i, arr) {
    i % 2 === 0
      ? (arr[i] = el.slice(1))
      : (res += '<span class="' + colours[arr[i - 1]] + '">' + el + "</span>");
  });

  return res;
}

export default function Colourify(props) {
  return (
    <span
      dangerouslySetInnerHTML={{ __html: colourise(sanitize(props.string)) }}
    />
  );
}

