package com.rapz.n3xr;

import java.util.Arrays;
import java.util.Random;

public final class BenchMain {
    private BenchMain() {}

    public static void main(String[] args) {
        System.out.println("native available: " + NativeBridge.isAvailable());
        if (!NativeBridge.isAvailable()) {
            fail(1, "library native tidak termuat");
        }
        if (!NativeBridge.init()) {
            fail(1, "nativeInit gagal");
        }
        String version = NativeBridge.version();
        System.out.println("version: " + version);
        if (!"1.0.0".equals(version)) {
            fail(1, "versi tidak sesuai, dapat: " + version);
        }

        // 1. Hasil native harus sama persis dengan Java.
        int count = 8;
        int[] ids = new int[count * FaceMasker.SECTION_VOLUME];
        Random r = new Random(1);
        for (int i = 0; i < ids.length; i++) ids[i] = r.nextInt(4);
        byte[] flags = {0, 3, 3, 1};

        byte[] javaMask = FaceMasker.buildJava(ids, flags, count);
        byte[] nativeMask = FaceMasker.buildNative(ids, flags, count);
        if (!Arrays.equals(javaMask, nativeMask)) {
            fail(2, "mask native BEDA dengan mask Java");
        }
        System.out.println("mask native == java: OK");

        // 2. Jalur mesh tidak boleh melempar.
        SectionMesher.MeshData[] meshes = SectionMesher.build(ids, flags, count);
        int faces = 0;
        for (SectionMesher.MeshData m : meshes) faces += m.faceCount();
        System.out.println("mesh faces total: " + faces);

        // 3. Input salah harus ditolak dengan exception, bukan crash.
        try {
            FaceMaskBridge.nativeBuildFaceMasks(new int[10], flags, 1);
            fail(3, "input salah tidak ditolak");
        } catch (IllegalArgumentException expected) {
            System.out.println("input salah ditolak: OK");
        }

        // 4. Benchmark.
        FaceMasker.benchmark(64, 20);
        System.out.println("SEMUA TES LOLOS");
    }

    private static void fail(int code, String msg) {
        System.out.println("GAGAL: " + msg);
        System.exit(code);
    }
}