using System;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

public static class LBM
{
    private static IntPtr ResultsPtr;

    [DllImport("lbmaf")]
    private static extern IntPtr init_array_ptr();

    [DllImport("lbmaf")]
    private static extern IntPtr computation_slow();

    // [DllImport("lbmaf")]
    // private static extern void cleanup(IntPtr pointer);

    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    public static void Compute()
    {
        ResultsPtr = computation_slow();
    }

    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    public static IntPtr GetPtr()
    {
        return ResultsPtr;
    }

    // [MethodImpl(MethodImplOptions.AggressiveInlining)]
    // public static void Cleanup()
    // {
    //     cleanup(ResultsPtr);
    // }
}
