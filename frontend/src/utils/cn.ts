// SPDX-License-Identifier: AGPL-3.0-only
// Copyright Authors of kriger

import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export default function cn(...args: ClassValue[]) {
  return twMerge(clsx(args));
}