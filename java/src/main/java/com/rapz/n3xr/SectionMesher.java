package com.rapz.n3xr;

import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.util.Arrays;

/** Membuat mesh section. Memakai Rust jika tersedia, selain itu fallback Java. */
public final class SectionMesher {
    public static final int SECTION_SIZE = 16;
    public static final int SECTION_VOLUME = 4096;
    public static final byte FLAG_VISIBLE = 1;
    public static final byte FLAG_OPAQUE = 2;

    /** Hasil satu section. Vertex 8 byte: x,y,z,face,corner,pad (u8) + blockId (u16 LE). */
    public static final class MeshData {
        public final int vertexCount;
        public final byte[] vertices;
        public final int[] indices;

        MeshData(int vertexCount, byte[] vertices, int[] indices) {
            this.vertexCount = vertexCount;
            this.vertices = vertices;
            this.indices = indices;
        }

        public int faceCount() {
            return indices.length / 6;
        }
    }

    private static final int[][] DIRS = {
        {-1, 0, 0}, {1, 0, 0}, {0, -1, 0}, {0, 1, 0}, {0, 0, -1}, {0, 0, 1}
    };
    private static final int[][][] CORNERS = {
        {{0, 0, 0}, {0, 0, 1}, {0, 1, 1}, {0, 1, 0}},
        {{1, 0, 1}, {1, 0, 0}, {1, 1, 0}, {1, 1, 1}},
        {{0, 0, 0}, {1, 0, 0}, {1, 0, 1}, {0, 0, 1}},
        {{0, 1, 1}, {1, 1, 1}, {1, 1, 0}, {0, 1, 0}},
        {{1, 0, 0}, {0, 0, 0}, {0, 1, 0}, {1, 1, 0}},
        {{0, 0, 1}, {1, 0, 1}, {1, 1, 1}, {0, 1, 1}}
    };

    private SectionMesher() {}

    /**
     * Membuat mesh untuk {@code count} section. {@code ids} panjangnya count * 4096,
     * urutan indeks y*256 + z*16 + x. Native dicoba dulu; jika gagal, fallback Java.
     */
    public static MeshData[] build(int[] ids, byte[] flags, int count) {
        validate(ids, flags, count);
        if (NativeBridge.isAvailable() && NativeBridge.init()) {
            try {
                byte[] raw = NativeBridge.buildSections(ids, flags, count);
                if (raw != null) {
                    return parse(raw, count);
                }
            } catch (RuntimeException | UnsatisfiedLinkError e) {
                // Jatuh ke fallback Java supaya game tetap berjalan.
            }
        }
        return buildJava(ids, flags, count);
    }

    private static void validate(int[] ids, byte[] flags, int count) {
        if (ids == null || flags == null) {
            throw new IllegalArgumentException("ids/flags must not be null");
        }
        if (count <= 0 || ids.length != count * SECTION_VOLUME) {
            throw new IllegalArgumentException("ids length must be count * " + SECTION_VOLUME);
        }
    }

    private static MeshData[] parse(byte[] raw, int count) {
        ByteBuffer buf = ByteBuffer.wrap(raw).order(ByteOrder.LITTLE_ENDIAN);
        MeshData[] result = new MeshData[count];
        for (int s = 0; s < count; s++) {
            if (buf.remaining() < 8) {
                throw new IllegalStateException("truncated native mesh data");
            }
            int vc = buf.getInt();
            int ic = buf.getInt();
            if (vc < 0 || ic < 0 || buf.remaining() < (long) vc * 8 + (long) ic * 4) {
                throw new IllegalStateException("corrupt native mesh data");
            }
            byte[] verts = new byte[vc * 8];
            buf.get(verts);
            int[] idx = new int[ic];
            for (int i = 0; i < ic; i++) idx[i] = buf.getInt();
            result[s] = new MeshData(vc, verts, idx);
        }
        return result;
    }

    // ---------------- Fallback Java (tanpa section tetangga, sama dengan jalur JNI v1) ---------

    private static MeshData[] buildJava(int[] ids, byte[] flags, int count) {
        MeshData[] result = new MeshData[count];
        for (int s = 0; s < count; s++) {
            result[s] = meshOne(ids, s * SECTION_VOLUME, flags);
        }
        return result;
    }

    private static int flagOf(byte[] flags, int id) {
        if (id < 0 || id >= flags.length) return 0;
        return flags[id] & 0xFF;
    }

    private static MeshData meshOne(int[] ids, int off, byte[] flags) {
        ByteBuffer verts = ByteBuffer.allocate(8 * 64).order(ByteOrder.LITTLE_ENDIAN);
        int[] idx = new int[64];
        int vcount = 0;
        int icount = 0;

        for (int y = 0; y < SECTION_SIZE; y++) {
            for (int z = 0; z < SECTION_SIZE; z++) {
                for (int x = 0; x < SECTION_SIZE; x++) {
                    int id = ids[off + y * 256 + z * 16 + x];
                    if (id < 0 || id > 0xFFFF) {
                        throw new IllegalArgumentException("block id out of range: " + id);
                    }
                    if ((flagOf(flags, id) & FLAG_VISIBLE) == 0) continue;

                    for (int f = 0; f < 6; f++) {
                        int nx = x + DIRS[f][0];
                        int ny = y + DIRS[f][1];
                        int nz = z + DIRS[f][2];
                        boolean inside = nx >= 0 && nx < 16 && ny >= 0 && ny < 16 && nz >= 0 && nz < 16;
                        if (inside) {
                            int nid = ids[off + ny * 256 + nz * 16 + nx];
                            if ((flagOf(flags, nid) & FLAG_OPAQUE) != 0 || nid == id) continue;
                        }

                        if (verts.remaining() < 32) {
                            ByteBuffer bigger = ByteBuffer.allocate(verts.capacity() * 2)
                                .order(ByteOrder.LITTLE_ENDIAN);
                            verts.flip();
                            bigger.put(verts);
                            verts = bigger;
                        }
                        if (icount + 6 > idx.length) {
                            idx = Arrays.copyOf(idx, idx.length * 2);
                        }

                        int base = vcount;
                        for (int c = 0; c < 4; c++) {
                            verts.put((byte) (x + CORNERS[f][c][0]));
                            verts.put((byte) (y + CORNERS[f][c][1]));
                            verts.put((byte) (z + CORNERS[f][c][2]));
                            verts.put((byte) f);
                            verts.put((byte) c);
                            verts.put((byte) 0);
                            verts.putShort((short) id);
                            vcount++;
                        }
                        idx[icount++] = base;
                        idx[icount++] = base + 1;
                        idx[icount++] = base + 2;
                        idx[icount++] = base;
                        idx[icount++] = base + 2;
                        idx[icount++] = base + 3;
                    }
                }
            }
        }
        return new MeshData(vcount, Arrays.copyOf(verts.array(), vcount * 8), Arrays.copyOf(idx, icount));
    }
}