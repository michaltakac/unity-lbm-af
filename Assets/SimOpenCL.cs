using System;
using System.Collections;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using UnityEngine;

public class SimOpenCL : MonoBehaviour
{
    [DllImport("lbmaf")]
    static extern void init_array_opencl(IntPtr texture);

	private Texture2D tex;
    private Material material;

	void Start()
	{
		tex = new Texture2D(128, 128, TextureFormat.RGBA32, false);
		material = GetComponent<Renderer>().material;
        material.mainTexture = tex;

		GetComponent<Renderer>().material.mainTexture = image;

        Debug.Log("Simulation started!");
        init_array_opencl(tex.GetNativeTexturePtr());
	}
	
	void Update()
	{
		
	}
}
