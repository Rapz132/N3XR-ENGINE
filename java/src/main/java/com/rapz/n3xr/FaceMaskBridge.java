package com.rapz.n3xr;

/** Deklarasi native untuk mask face. Pakai lewat FaceMasker, jangan langsung. */
final class FaceMaskBridge {
    private FaceMaskBridge() {}

    static native byte[] nativeBuildFaceMasks(int[] ids, byte[] flags, int count);
}