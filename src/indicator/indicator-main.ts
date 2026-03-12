import IndicatorApp from "./IndicatorApp.svelte";
import { mount } from "svelte";

const app = mount(IndicatorApp, { target: document.getElementById("indicator")! });

export default app;
